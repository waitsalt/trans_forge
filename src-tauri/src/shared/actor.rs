use anyhow::{Result, anyhow};
use std::{fmt, future::Future, time::Duration};
use tauri::async_runtime;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

const BATCH_SIZE: usize = 16;

pub trait Actor: Send + Sized + 'static {
    type Message: Send + 'static;

    fn handle(
        &mut self,
        msg: Self::Message,
        ctx: &Context<Self>,
    ) -> impl Future<Output = Result<()>> + Send;

    fn on_start(&mut self, _ctx: &Context<Self>) -> impl Future<Output = Result<()>> + Send {
        async {
            tracing::trace!("Actor 启动");
            Ok(())
        }
    }

    fn on_stop(&mut self, _ctx: &Context<Self>) -> impl Future<Output = Result<()>> + Send {
        async {
            tracing::trace!("Actor 停止");
            Ok(())
        }
    }

    fn start(mut self, size: usize) -> Addr<Self> {
        let (tx, mut rx) = mpsc::channel(size);
        let cancel_token = CancellationToken::new();
        let token_for_task = cancel_token.clone();

        let addr = Addr {
            sender: tx.clone(),
            cancel_token: cancel_token.clone(),
        };

        async_runtime::spawn(async move {
            let context = Context { addr };

            if let Err(e) = self.on_start(&context).await {
                tracing::error!(%e, "Actor 启动时发生错误");
                return;
            }

            let mut buffer = Vec::with_capacity(BATCH_SIZE);
            let mut should_exit = false;

            loop {
                tokio::select! {
                    biased;

                    _ = token_for_task.cancelled() => {
                        tracing::trace!("Actor 收到停止信号");
                        break;
                    }
                    count = rx.recv_many(&mut buffer, BATCH_SIZE) => {
                        if count == 0 {
                            tracing::trace!("Actor 消息通道已关闭");
                            should_exit = true;
                        }

                        for msg in buffer.drain(..) {
                            if token_for_task.is_cancelled() {
                                break;
                            }

                            if let Err(e) = self.handle(msg, &context).await {
                                tracing::error!(%e, "Actor 处理消息错误");
                            }
                        }

                        if should_exit {
                            break;
                        }
                    }
                }
            }

            if let Err(e) = self.on_stop(&context).await {
                tracing::error!(%e, "Actor 停止时发生错误");
            }
        });

        Addr {
            sender: tx,
            cancel_token,
        }
    }
}

pub struct Addr<A: Actor> {
    sender: mpsc::Sender<A::Message>,
    cancel_token: CancellationToken,
}

impl<A: Actor> fmt::Debug for Addr<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Addr")
            .field(
                "sender",
                &format_args!("mpsc::Sender<{}>", std::any::type_name::<A::Message>()),
            )
            .field("cancel_token", &self.cancel_token)
            .finish()
    }
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            cancel_token: self.cancel_token.clone(),
        }
    }
}

#[allow(dead_code)]
impl<A> Addr<A>
where
    A: Actor,
{
    pub async fn send(&self, msg: A::Message) -> Result<(), mpsc::error::SendError<A::Message>> {
        self.sender.send(msg).await
    }

    pub fn try_send(&self, msg: A::Message) -> Result<(), mpsc::error::TrySendError<A::Message>> {
        self.sender.try_send(msg)
    }

    pub async fn ask<R, F>(&self, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(oneshot::Sender<R>) -> A::Message,
    {
        let (tx, rx) = oneshot::channel();
        self.send(f(tx))
            .await
            .map_err(|e| anyhow!("Actor 发送消息失败: {:#}", e))?;
        rx.await
            .map_err(|e| anyhow!("Actor 等待响应失败(Actor可能已丢弃请求): {:#}", e))
    }

    pub async fn ask_timeout<R, F>(&self, f: F, timeout: Duration) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(oneshot::Sender<R>) -> A::Message,
    {
        tokio::time::timeout(timeout, self.ask(f))
            .await
            .map_err(|_| anyhow!("Actor 请求超时"))?
    }

    pub fn stop(&self) {
        self.cancel_token.cancel();
    }

    pub fn is_stopped(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    pub fn downgrade(&self) -> WeakAddr<A> {
        WeakAddr {
            sender: self.sender.downgrade(),
            cancel_token: self.cancel_token.clone(),
        }
    }
}

#[allow(dead_code)]
pub struct WeakAddr<A: Actor> {
    sender: mpsc::WeakSender<A::Message>,
    cancel_token: CancellationToken,
}

impl<A: Actor> fmt::Debug for WeakAddr<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakAddr")
            .field(
                "sender",
                &format_args!("mpsc::WeakSender<{}>", std::any::type_name::<A::Message>()),
            )
            .field("cancel_token", &self.cancel_token)
            .finish()
    }
}

impl<A: Actor> Clone for WeakAddr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            cancel_token: self.cancel_token.clone(),
        }
    }
}

#[allow(dead_code)]
impl<A: Actor> WeakAddr<A> {
    pub fn upgrade(&self) -> Option<Addr<A>> {
        self.sender.upgrade().map(|sender| Addr {
            sender,
            cancel_token: self.cancel_token.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Context<A>
where
    A: Actor,
{
    addr: Addr<A>,
}

impl<A> Context<A>
where
    A: Actor,
{
    pub fn addr(&self) -> &Addr<A> {
        &self.addr
    }
}

impl<A: Actor> Clone for Context<A> {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}

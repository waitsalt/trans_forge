// 在 Windows 的 release 模式下阻止额外控制台窗口弹出，请勿删除！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tran_001_lib::run()
}

// This file is auto-generated and managed by the flowlang build script.
use flowlang::rustcmd::Transform;
pub mod restart_service;
pub mod stop_service;
pub mod start_service;
pub mod uninstall_service;
pub mod install_service;
pub mod install_release;
pub mod info;
pub mod members;
pub mod remove_member;
pub mod add_member;
pub mod join_network;
pub mod create_network;
pub mod build_config;
pub mod save_config;
pub fn cmdinit(cmds: &mut Vec<(String, Transform, String)>) {
    cmds.push(("ytxwzx19fe76e2c71r1".to_string(), save_config::execute, "".to_string()));
    cmds.push(("qxszhx19fe76e6afcj3".to_string(), build_config::execute, "".to_string()));
    cmds.push(("oxhyyu19fe76e6b01n5".to_string(), create_network::execute, "".to_string()));
    cmds.push(("qhjshv19fe76e6b07o7".to_string(), join_network::execute, "".to_string()));
    cmds.push(("zimlkq19fe76e6b0cn9".to_string(), add_member::execute, "".to_string()));
    cmds.push(("hlpkqi19fe76e6b10zb".to_string(), remove_member::execute, "".to_string()));
    cmds.push(("hqkmoj19fe76e6b16wd".to_string(), members::execute, "".to_string()));
    cmds.push(("wuggps19fe76e6b1avf".to_string(), info::execute, "".to_string()));
    cmds.push(("vwojjn19fe76e6b1fr11".to_string(), install_release::execute, "".to_string()));
    cmds.push(("qyokmt19fe76e6b24j13".to_string(), install_service::execute, "".to_string()));
    cmds.push(("zjlhwx19fe76e6b29o15".to_string(), uninstall_service::execute, "".to_string()));
    cmds.push(("zwpkoo19fe76e6b2ei17".to_string(), start_service::execute, "".to_string()));
    cmds.push(("gorqly19fe76e6b33i19".to_string(), stop_service::execute, "".to_string()));
    cmds.push(("uhtkqq19fe76e6b38t1b".to_string(), restart_service::execute, "".to_string()));
}

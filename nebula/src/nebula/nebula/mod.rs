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
pub mod start;
pub mod init;
pub fn cmdinit(cmds: &mut Vec<(String, Transform, String)>) {
    cmds.push(("rvhgkr190c3084286y3de".to_string(), init::execute, "".to_string()));
    cmds.push(("hpsujs190c1e02ff6o143".to_string(), start::execute, "".to_string()));
    cmds.push(("pyzxrl19ff06c2457k1".to_string(), save_config::execute, "".to_string()));
    cmds.push(("igykzi19ff06c57eel3".to_string(), build_config::execute, "".to_string()));
    cmds.push(("wksoog19ff06c5920t5".to_string(), create_network::execute, "".to_string()));
    cmds.push(("mhmtmw19ff06c5a54m7".to_string(), join_network::execute, "".to_string()));
    cmds.push(("jompkl19ff06c5b79t9".to_string(), add_member::execute, "".to_string()));
    cmds.push(("porjhm19ff06c5c9atb".to_string(), remove_member::execute, "".to_string()));
    cmds.push(("tikzwu19ff06c5dbaxd".to_string(), members::execute, "".to_string()));
    cmds.push(("llixku19ff06c5ee6kf".to_string(), info::execute, "".to_string()));
    cmds.push(("hnoyro19ff06c6009j11".to_string(), install_release::execute, "".to_string()));
    cmds.push(("vopmor19ff06c652aj13".to_string(), install_service::execute, "".to_string()));
    cmds.push(("hlnukj19ff06c6a9fg15".to_string(), uninstall_service::execute, "".to_string()));
    cmds.push(("ngvkoz19ff06c6aa4i17".to_string(), start_service::execute, "".to_string()));
    cmds.push(("zsogsu19ff06c6aa8x19".to_string(), stop_service::execute, "".to_string()));
    cmds.push(("hksqzt19ff06c6aadg1b".to_string(), restart_service::execute, "".to_string()));
}

#[macro_use] extern crate log;
use pretty_env_logger;
use std::time::Duration;
use signal_hook::iterator::Signals;
use signal_hook::{SIGTERM,SIGINT};

mod settings;
mod service;

use nexus_unity_sdbp::datatypes::*;
use std::thread::sleep;
use nexus_unity_sdbp::drv::core::{DeviceFilter, SharedStats, Stats, DeviceHandler, Dispatcher, Controller, DrvMeta, UdsServer, SdbpkCheck};
use std::process::exit;


fn start_main() {

    //env::set_var("RUST_APP_LOG", "debug");
    //env::set_var("RUST_BACKTRACE", "1");

    pretty_env_logger::init_custom_env("RUST_APP_LOG");
    let version = env!("CARGO_PKG_VERSION");

    let check = SdbpkCheck {
        major: 1,
        minor: 1,
        patch: 0
    };

    match check.check_version() {
        Ok(version) => {
            info!("SDBPK driver version: {}.{}.{}", version.major, version.minor, version.patch)
        }
        Err(err) => {
            error!("{}",err);
            exit(-1)
        }
    }

    info!("Module driver version: {}",version);

    let signals = Signals::new(&[SIGTERM,SIGINT]).ok().unwrap();
    let mut filter  = DeviceFilter::<String>::new();
    filter.add(settings::MODULE_NAME.to_string());

    /*
     * Prepare Global Settings
     */
    let shared = SharedStats::new(Stats::new(settings::MODULE_NAME.to_string(),Version::from_str(version).unwrap(),check.to_version()));

    /*
     * Device-Event channels
     */
    let (devt_sender,devt_receiver) = crossbeam_channel::unbounded();

    let device_handler = DeviceHandler::start(filter,devt_receiver.clone(),devt_sender.clone());
    let dispatcher = Dispatcher::start();
    let controller = Controller::start(dispatcher.get_com(), devt_receiver.clone(), shared.clone(), service::PowerModule::handle_function);

    let meta = DrvMeta::new(settings::MODULE_NAME.to_string(),settings::DRV_NAME.to_string(),settings::SOCKET_PATH.to_string());
    let udsserver = UdsServer::start(meta,dispatcher.get_com(),shared.clone());

    info!("Started driver for {}",settings::MODULE_NAME);

    for _sig in signals.forever() {
        controller.stop(Duration::from_millis(1000));
        dispatcher.stop(Duration::from_millis(1000));
        device_handler.stop(Duration::from_millis(1000));
        udsserver.stop(Duration::from_millis(10000));
        break;
    }

    sleep(Duration::from_secs(1));

}


fn main() {
    start_main();
}

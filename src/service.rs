use std::time::{Duration};

use nexus_unity_sdbp::datatypes::Descriptor;
use nexus_unity_sdbp::util::*;
use nexus_unity_sdbp::drv::core::*;
use nexus_unity_sdbp::sdbp::*;

pub struct PowerModule {}

impl PowerModule {


    fn transfer(dev_handle : &mut DeviceHandle, buf: Vec<u8>) -> Result<Vec<u8>,std::io::Error>{

        let res = dev_handle.write(buf);
        if res.is_err() {
            return Err(res.unwrap_err());
        }

        let mut response = vec![0; 4096];
        let res = dev_handle.read(&mut response);
        match res {
            Ok(value) => return Ok(Vec::from(&response[0..value])),
            Err(_err) => return Err(_err),
        };
    }

    pub fn handle_function( desc : Descriptor, ctl_pair : ChannelPair<ManagedThreadState>, dev_pair : ChannelPair<PMsg>) {

        let mut stopped = false;
        let mut err_cnt : u32 = 0;
        debug!("Started {} for {}" ,std::thread::current().name().unwrap(),desc.path().to_str().unwrap());


        while !stopped {
            ManagedThreadUtil::is_stopped(&mut stopped,&ctl_pair);


            //Init Sequence
            let result  = DeviceHandle::new(&desc.dev_file());
            let mut dev_handle = match result {
                None => {
                    trace!("{:?} - Cannot open device file", desc.dev_file());
                    continue;
                },
                Some(value) => value,
            };

            info!("Setting communication speed to: {} kHz",desc.max_sclk_speed());
            match PowerModule::transfer(&mut dev_handle, CoreBuilder::new().control().set_sclk_speed(desc.max_sclk_speed()).unwrap()) {
                Ok(response) => {
                    if response[0] != 0x01 || response[1] != 0x03 || response[2] != 0x08 || response[3] != 0x00 {
                        panic!("Communication speed change failed")
                    }
                },
                Err(_) => { panic!("Failed setting communication speed") },
            };


            while !stopped {
                ManagedThreadUtil::is_stopped(&mut stopped,&ctl_pair);
                let result =dev_pair.rx().recv_timeout(Duration::from_millis(150));
                if  result.is_ok(){

                    let msg= result.unwrap();
                    trace!("{:?} - rx - {:?}",&desc.path().to_str().unwrap(),msg);

                    //if(msg.get_msg() == sdbp::CoreBuilder::new())

                    let mut response = Err(std::io::Error::from(std::io::ErrorKind::NotConnected));
                    for i in 0..3 {

                        let ret =  PowerModule::transfer(&mut dev_handle,msg.get_msg().unwrap());

                        if ret.is_err() {
                            if i == 2 {
                                response = ret;
                            }
                            err_cnt+=1;
                        }else {
                            response = ret;
                            break;
                        }
                    }
                    let answer = PMsg::create(msg.get_dst(), msg.get_src(),response );
                    trace!("{:?} - tx - {:?}",&desc.path().to_str().unwrap(),msg);
                    let _ = dev_pair.tx().send(answer);

                }

                match PowerModule::transfer(&mut dev_handle,FrameBuilder::new().core().control().mode_run().unwrap()) {
                    Err(_err) => { err_cnt += 1; trace!("Error cnt: {}", err_cnt); break},
                    Ok(value) => value,
                };
            }
            drop(dev_handle);

        }
        debug!("Stopped {}",std::thread::current().name().unwrap());
    }
}
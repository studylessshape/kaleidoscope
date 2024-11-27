use llvm_sys::{
    core::LLVMCreatePassManager,
    prelude::LLVMPassManagerRef,
    target_machine::{LLVMCreateTargetMachineOptions, LLVMTargetMachineOptionsRef},
    transforms::pass_builder::*,
};

use crate::bool_to_llvm;

pub struct PassManager {
    pass_manager: LLVMPassManagerRef,
    options: LLVMPassBuilderOptionsRef,
    machine_options: LLVMTargetMachineOptionsRef,
}

macro_rules! impl_enable_for_builder_options {
    ($($func_name:ident => $target:path),+) => {
        $(
            pub fn $func_name(&self, enable: bool) -> &Self {
                unsafe {
                    $target(self.options, bool_to_llvm(enable));
                }

                self
            }
        )+
    };
}

impl PassManager {
    pub fn new() -> Self {
        unsafe {
            Self {
                pass_manager: LLVMCreatePassManager(),
                options: LLVMCreatePassBuilderOptions(),
                machine_options: LLVMCreateTargetMachineOptions(),
            }
        }
    }

    impl_enable_for_builder_options!(
        debug_logging => LLVMPassBuilderOptionsSetDebugLogging,
        call_graph_profile => LLVMPassBuilderOptionsSetCallGraphProfile,
        loop_interleaving => LLVMPassBuilderOptionsSetLoopInterleaving,
        loop_vectorization => LLVMPassBuilderOptionsSetLoopVectorization,
        slp_vectorization => LLVMPassBuilderOptionsSetSLPVectorization,
        loop_unrolling => LLVMPassBuilderOptionsSetLoopUnrolling,
        forget_all_scev_in_loop_unroll => LLVMPassBuilderOptionsSetForgetAllSCEVInLoopUnroll
    );
}

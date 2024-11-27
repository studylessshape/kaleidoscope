use llvm_sys::{
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMLinkInMCJIT,
    },
    orc2::{
        ee::LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager,
        lljit::{
            LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcLLJITBuilderSetJITTargetMachineBuilder, LLVMOrcLLJITGetExecutionSession,
            LLVMOrcLLJITGetGlobalPrefix, LLVMOrcLLJITMangleAndIntern, LLVMOrcLLJITRef,
        },
        LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess,
        LLVMOrcDefinitionGeneratorRef, LLVMOrcExecutionSessionCreateBareJITDylib, LLVMOrcExecutionSessionRef,
        LLVMOrcJITDylibAddGenerator, LLVMOrcJITDylibRef,
        LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine,
        LLVMOrcObjectLayerRef, LLVMOrcSymbolStringPoolEntryRef,
    },
    prelude::LLVMModuleRef,
    target::{LLVMTargetDataRef, LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout, LLVMCreateTargetMachine,
        LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures, LLVMGetHostCPUName,
        LLVMGetTargetFromTriple, LLVMNormalizeTargetTriple, LLVMRelocMode, LLVMTargetMachineRef,
        LLVMTargetRef,
    },
};
use std::{
    ffi::CString,
    io::Error as IoError,
    mem,
    ptr::null_mut,
};

use crate::{get_error_msg, LLVM_SUCCESS};

pub struct KaleicoscopeJit {
    execution_session: LLVMOrcExecutionSessionRef,
    target_machine: LLVMTargetMachineRef,
    data_layout: LLVMTargetDataRef,
    jit: LLVMOrcLLJITRef,
    object_layer: LLVMOrcObjectLayerRef,
    main_jd: LLVMOrcJITDylibRef,
    mangle_and_interner: LLVMOrcSymbolStringPoolEntryRef,
}

impl KaleicoscopeJit {
    pub fn create() -> Result<Self, IoError> {
        unsafe {
            let target_machine = Self::create_tm()?;
            // let jtmb = LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine(target_machine);

            let data_layout = LLVMCreateTargetDataLayout(target_machine);
            let jit = Self::create_jit(target_machine)?;
            let es = LLVMOrcLLJITGetExecutionSession(jit);
            let object_layer = LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager(es);

            let main_jd_name = CString::new("<main>").unwrap();
            let main_jd = LLVMOrcExecutionSessionCreateBareJITDylib(es, main_jd_name.as_ptr());

            LLVMOrcJITDylibAddGenerator(main_jd, Self::create_generator(jit)?);

            let mangle_name = CString::new("mangle").unwrap();
            let mangle_and_interner = LLVMOrcLLJITMangleAndIntern(jit, mangle_name.as_ptr());

            Ok(Self {
                execution_session: es,
                data_layout,
                jit,
                target_machine,
                object_layer,
                main_jd,
                mangle_and_interner,
            })
        }
    }

    unsafe fn get_target(triple: *const ::libc::c_char) -> Result<LLVMTargetRef, IoError> {
        let mut target = mem::MaybeUninit::uninit();
        let mut error = mem::zeroed();
        if LLVMGetTargetFromTriple(triple, target.as_mut_ptr(), &mut error) != LLVM_SUCCESS {
            if !error.is_null() {
                return Err(IoError::other(CString::from_raw(error).to_string_lossy()));
            }
        }

        Ok(target.assume_init())
    }

    unsafe fn create_tm() -> Result<LLVMTargetMachineRef, IoError> {
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();

        let triple = LLVMNormalizeTargetTriple(LLVMGetDefaultTargetTriple());
        let target = Self::get_target(triple)?;
        let cpu = LLVMGetHostCPUName();
        let features = LLVMGetHostCPUFeatures();

        let tm = LLVMCreateTargetMachine(
            target,
            triple,
            cpu,
            features,
            LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        if tm.is_null() {
            panic!("Target machine is null!");
        }

        Ok(tm)
    }

    unsafe fn create_jit(tm: LLVMTargetMachineRef) -> Result<LLVMOrcLLJITRef, IoError> {
        let jit_target_machine_builder = LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine(tm);

        let jit_builder = LLVMOrcCreateLLJITBuilder();
        LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(jit_builder, jit_target_machine_builder);

        let mut jit_ref = mem::MaybeUninit::uninit();
        let err = LLVMOrcCreateLLJIT(jit_ref.as_mut_ptr(), jit_builder);
        if !err.is_null() {
            return Err(IoError::other(get_error_msg(err)));
        }

        Ok(jit_ref.assume_init())
    }

    unsafe fn create_execution_engine(
        module: LLVMModuleRef,
    ) -> Result<LLVMExecutionEngineRef, IoError> {
        let mut ee_ref = mem::MaybeUninit::uninit();
        let mut error = mem::zeroed();

        if LLVMCreateExecutionEngineForModule(ee_ref.as_mut_ptr(), module, &mut error)
            != LLVM_SUCCESS
        {
            if !error.is_null() {
                return Err(IoError::other(CString::from_raw(error).to_string_lossy()));
            }
        }

        Ok(ee_ref.assume_init())
    }

    unsafe fn create_generator(
        jit: LLVMOrcLLJITRef,
    ) -> Result<LLVMOrcDefinitionGeneratorRef, IoError> {
        let mut generator = mem::MaybeUninit::uninit();
        let error = LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess(
            generator.as_mut_ptr(),
            LLVMOrcLLJITGetGlobalPrefix(jit),
            None,
            null_mut(),
        );
        if !error.is_null() {
            Err(IoError::other(get_error_msg(error)))
        } else {
            Ok(generator.assume_init())
        }
    }
}

#[cfg(test)]
mod test {
    use super::KaleicoscopeJit;

    #[test]
    fn test_method() {
        let jit = KaleicoscopeJit::create();
        match jit {
            Ok(_) => assert!(true),
            Err(e) => panic!("{}", e),
        }
    }
}

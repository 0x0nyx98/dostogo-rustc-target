use crate::spec::{base, PanicStrategy, Target, TargetMetadata};

pub fn target() -> Target {
    let mut base = base::msdos6::opts();
    base.cpu = "i686".into();
    base.disable_redzone = true;
    base.panic_strategy = PanicStrategy::Abort;
    base.features = "-mmx,-sse,+soft-float".into();

    Target {
        llvm_target: "i686-unknown-none".into(),
        pointer_width: 32,
        data_layout: "e-m:x-p:32:32-p270:32:32-p271:32:32-p272:64:64-\
            i64:64-i128:128-f80:32-n8:16:32-a:0:32-S32"
            .into(),
        arch: "x86".into(),
        options: base,
        metadata: TargetMetadata {
            description: None,
            tier: None,
            host_tools: None,
            std: None,
        },
    }
}
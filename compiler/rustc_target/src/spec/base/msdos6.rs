use crate::spec::crt_objects;
use crate::spec::{Cc, LinkerFlavor, Lld, RelocModel, StackProbeType, TargetOptions};

pub fn opts() -> TargetOptions {
    TargetOptions {
        os: "MS-DOS".into(),
        linker: Some("rust-lld".into()),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        stack_probes: StackProbeType::Inline,
        relocation_model: RelocModel::Static,
        //pre_link_objects: crt_objects::pre_msdos6(),
        //post_link_objects: crt_objects::post_msdos6(),
        ..Default::default()
    }
}

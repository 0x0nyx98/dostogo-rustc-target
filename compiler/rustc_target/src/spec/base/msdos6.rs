//use crate::spec::crt_objects;
use crate::spec::{Cc, LinkerFlavor, Lld, RelocModel, StackProbeType, TargetOptions};

pub(crate) fn opts() -> TargetOptions {
    TargetOptions {
        os: "msdos6".into(), // omg i thought this was like a flavor text thing when i wrote it so i set it to "MS-DOS". its used for target_os. future onnie pls dont touch this
        linker: Some("rust-lld".into()),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        stack_probes: StackProbeType::Inline,
        relocation_model: RelocModel::Static,
        //pre_link_objects: crt_objects::pre_msdos6(),
        //post_link_objects: crt_objects::post_msdos6(),
        ..Default::default()
    }
}

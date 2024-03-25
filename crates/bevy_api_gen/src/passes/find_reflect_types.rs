use log::debug;
use rustc_hir::def_id::LOCAL_CRATE;

use crate::{Args, BevyCtxt, ReflectType, DEF_PATHS_REFLECT};

/// Finds all reflect types which we can wrap in the crate as well as sorts the final list.
pub(crate) fn find_reflect_types(ctxt: &mut BevyCtxt<'_>, args: &Args) -> bool {
    let tcx = &ctxt.tcx;

    for trait_did in tcx.all_local_trait_impls(()).keys() {
        // we want to find the canonical `Reflect` trait's implemenations across crates, so let's check all impls and choose those
        // whose def_path is equal to what we know the Reflect trait's is

        let def_path_str = tcx.def_path_str(trait_did);

        if !DEF_PATHS_REFLECT.contains(&def_path_str.as_str()) {
            continue;
        }

        debug!(
            "Found Reflect impls in crate: {}, with path: {}",
            tcx.crate_name(LOCAL_CRATE),
            def_path_str
        );

        // this returns non-local impls as well
        let reflect_trait_impls = tcx.trait_impls_of(trait_did);

        // blanket impls are implementations on generics directly, i.e. `impl From<T> for T`
        // non blanket impls may also contain generics but those will be contained within another type i.e. `impl Default for Vec<T>`
        // ignore anything with a generic, so blanket_impls are out for now
        // we also make sure to only work over types and impls directly in the local crate
        let reflect_adts_did = reflect_trait_impls
            .non_blanket_impls()
            .iter()
            .flat_map(|(self_ty, impl_dids)| impl_dids.iter().zip(std::iter::repeat(self_ty)))
            .filter_map(|(impl_did, self_ty)| {
                let generics = tcx.generics_of(*impl_did);
                (impl_did.is_local() &&
                    generics.own_counts().types == 0
                        && generics.own_counts().consts == 0
                        && generics.own_counts().lifetimes == 0
                        // only non parametrized simple types are allowed, i.e. "MyStruct" is allowed but "MyStruct<T>" isn't
                        // we also only have details about the local crates structs so skip non local types
                        && self_ty.def().is_some())
                .then(|| self_ty.def().unwrap())
            })
            .inspect(|impl_| debug!("On type: {:?}", tcx.item_name(*impl_)))
            .map(|did| (did, ReflectType::default()));

        ctxt.reflect_types.extend(reflect_adts_did);
    }

    ctxt.reflect_types
        .sort_by_cached_key(|did, _| did.index.as_u32() + did.krate.as_u32());

    if args.cmd.is_list_types() {
        for did in ctxt.reflect_types.keys() {
            println!("{:?}", tcx.def_path_str(did));
        }
        return false;
    }

    true
}

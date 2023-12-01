use indexmap::{IndexMap, IndexSet};
use rustdoc_types::{
    Crate, Enum, Id, Impl, Import, Item, ItemEnum, Module, Struct, Trait, Visibility,
};
use serde_derive::Serialize;
use std::borrow::Cow::{Borrowed, Owned};
use std::collections::hash_map::RandomState;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::iter::{once, repeat};
use std::{borrow::Cow, ops::Deref};

pub fn print_item_variant(variant: &ItemEnum) -> &'static str {
    match variant {
        ItemEnum::Module(_) => "Module",
        ItemEnum::ExternCrate { .. } => "ExternCrate",
        ItemEnum::Import(_) => "Import",
        ItemEnum::Union(_) => "Union",
        ItemEnum::Struct(_) => "Struct",
        ItemEnum::StructField(_) => "StructField",
        ItemEnum::Enum(_) => "Enum",
        ItemEnum::Variant(_) => "Variant",
        ItemEnum::Function(_) => "Function",
        ItemEnum::Trait(_) => "Trait",
        ItemEnum::TraitAlias(_) => "TraitAlias",
        ItemEnum::Impl(_) => "Impl",
        ItemEnum::TypeAlias(_) => "TypeAlias",
        ItemEnum::OpaqueTy(_) => "OpaqueTy",
        ItemEnum::Constant(_) => "Constant",
        ItemEnum::Static(_) => "Static",
        ItemEnum::ForeignType => "ForeignType",
        ItemEnum::Macro(_) => "Macro",
        ItemEnum::ProcMacro(_) => "ProcMacro",
        ItemEnum::Primitive(_) => "Primitive",
        ItemEnum::AssocConst { .. } => "AssocConst",
        ItemEnum::AssocType { .. } => "AssocType",
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct ImportPath {
    pub is_public: bool,
    pub components: Vec<String>,
}

impl ImportPath {
    pub fn new_public(value: Vec<String>) -> Self {
        Self {
            components: value,
            is_public: true,
        }
    }
}

impl Display for ImportPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.components.join("::").as_str())
    }
}

impl ImportPath {
    pub fn replace_prefix(&self, prefix: &str, replacement: &str) -> Self {
        let mut components = self.components.clone();
        if let Some(first) = components.first_mut() {
            if let Some(stripped) = first.strip_prefix(prefix) {
                *first = replacement.to_owned() + stripped;
            }
        }
        Self {
            components,
            is_public: self.is_public,
        }
    }
}

/// An Id which uniquely identifies a crate
#[derive(Clone, Eq, Copy)]
pub struct CrateId<'a>(&'a Crate, &'a str);
impl<'a> CrateId<'a> {
    pub fn crate_name(self) -> &'a str {
        self.1
    }
}

impl Debug for CrateId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CrateId").field(&self.1).finish()
    }
}

impl Deref for CrateId<'_> {
    type Target = Crate;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Hash for CrateId<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

impl PartialEq for CrateId<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

#[derive(Debug)]
pub struct CrawledImportData<'a> {
    /// contains import paths public + private of every item in impls and types
    /// as well as the traits implemented by impls, may contain MORE than that, but these should be ignored
    /// paths are sorted, with public and shorter paths appearing earlier
    paths: IndexMap<(CrateId<'a>, Id), Vec<ImportPath>>,
    /// a set of all impls in the crate, possibly for external traits for which paths will be present in a crate
    /// different from the definition crate
    impls: IndexSet<(CrateId<'a>, Id)>,
    /// a set of all structs/enums in the crate, always in their definition crates
    types: IndexSet<(CrateId<'a>, Id)>,
    /// A mapping from external traits in a local crate to another crate where the trait is defined
    ext_traits: IndexMap<(CrateId<'a>, Id), (CrateId<'a>, Id)>,
    all_crates: Vec<CrateId<'a>>,
}

impl CrawledImportData<'_> {
    pub fn get_types(&self) -> impl Iterator<Item = &(CrateId, Id)> {
        self.types.iter()
    }

    pub fn get_public_types(&self) -> impl Iterator<Item = &(CrateId, Id)> {
        self.get_types()
            .filter(|id| self.get_public_item_path(id).is_some())
    }

    pub fn get_impls(&self) -> impl Iterator<Item = &(CrateId, Id)> {
        self.impls.iter()
    }

    /// Searches for the given trait in both the given crate and resolved external traits list and returns the shortest path found (might not be public)
    pub fn get_trait_path<'a>(&'a self, trait_id: &(CrateId<'a>, Id)) -> Option<&ImportPath> {
        log::trace!(
            "Searching for trait path for trait: `{:?}` in crate: `{}`",
            &trait_id.1,
            trait_id.0.crate_name()
        );
        let out = self.get_item_path(trait_id).or_else(|| {
            log::trace!("Trait not in the given crate, searching external traits");
            self.ext_traits
                .get(trait_id)
                .and_then(|trait_def_id| self.get_item_path(trait_def_id))
        });
        log::trace!("Path found?: {:?}", out);
        out
    }

    pub fn get_public_trait_path<'a>(
        &'a self,
        trait_id: &(CrateId<'a>, Id),
    ) -> Option<&ImportPath> {
        self.get_trait_path(trait_id).filter(|p| p.is_public)
    }

    /// searches for the given item in the given crate and returns the shortest import path (might not be public)
    pub fn get_item_path<'a>(&'a self, id: &(CrateId<'a>, Id)) -> Option<&ImportPath> {
        self.paths.get(id).and_then(|paths| paths.first())
    }

    pub fn get_public_item_path<'a>(&'a self, id: &(CrateId<'a>, Id)) -> Option<&ImportPath> {
        self.get_item_path(id).filter(|p| p.is_public)
    }
}

/// Used to hold data for processing of import paths in rustdoc json output
/// Future proofed as it does not use the .paths data
#[derive(Default)]
pub struct ImportPathCrawler {
    data: IndexMap<String, CrateCrawlerData>,
}

#[derive(Default)]
pub(crate) struct CrateCrawlerData {
    /// The possible import paths for each leaf item
    pub(crate) paths: IndexMap<Id, Vec<Vec<Id>>>,
    /// All the impls in the crate
    pub(crate) impls: IndexSet<Id>,
    //// All the types in the crate
    pub(crate) types: IndexSet<Id>,
    /// Mapping from implementations to the traits they're implementing
    pub(crate) impls_to_traits: IndexMap<Id, Id>,
    /// Traits which were found with an external path which are being referenced
    /// in the impls, if not matched up to items from other crates, need to be removed together with impls which reference those
    pub(crate) external_traits: IndexMap<Id, Vec<String>>,
}

impl ImportPathCrawler {
    pub fn new() -> Self {
        Default::default()
    }

    /// Finalizes the crawler, once all items in every crate needed are crawled, calling this
    /// ensures that no items without a public import path are found in this struct and resolves cross-crate links
    ///
    /// crates must contain all of the crawled crates or the function will panic
    pub fn finalize(self, crates: &[Crate]) -> CrawledImportData {
        let crate_data_iter = self.data.iter().map(|(crate_id, data)| {
            (
                crates
                    .iter()
                    .find_map(|crate_| {
                        let ref_crate_id = crate_name(crate_);
                        if ref_crate_id == crate_id {
                            Some(CrateId(crate_, ref_crate_id))
                        } else {
                            None
                        }
                    })
                    .unwrap(),
                data,
            )
        });

        // before we process impls, we need to be able to resolve external items,
        // finalize all our paths and build a prefix tree from import paths to ID's so that we can resolve
        // external imports (which we only know in String form) to ID's + Crate ID's from the crate's we've crawled
        let mut paths = IndexMap::default();
        crate_data_iter
            .clone()
            .for_each(|(crate_id, data)| Self::finalize_paths(crate_id, data, &mut paths));
        paths.iter_mut().for_each(|(_, paths)| {
            // put shortest paths first, public ahead of private
            // we create a mapping from [1,inf) -> [1000, inf) then take away 1000 for public paths to shift this to [0,inf)
            // if we encounter longer paths, well somebody is doing something wrong but it ain't us
            paths.sort_by_key(|p| (p.components.len() * 1000) - (p.is_public as usize * 1000))
        });
        let mut tree = gtrie::Trie::<char, &(CrateId, Id)>::new();

        paths.iter().for_each(|(id, import_paths)| {
            for p in import_paths {
                // use full import syntax with ::'s so that My::Trait::Struct does not equal MyTrait::Struct
                tree.insert(p.to_string().chars(), id)
            }
        });

        // with the tree built we can process the rest
        let mut impls = IndexSet::default();
        let mut types = IndexSet::default();
        let mut ext_traits = IndexMap::default();
        crate_data_iter.clone().for_each(|(crate_id, data)| {
            Self::finalize_impls(crate_id, data, &mut impls, &mut ext_traits, &tree);
            Self::finalize_types(crate_id, data, &mut types);
        });
        CrawledImportData {
            paths,
            impls,
            types,
            ext_traits,
            all_crates: crate_data_iter.map(|(id, _)| id).collect(),
        }
    }

    /// Finalize the crawl data by converting search traces to import paths
    fn finalize_paths<'a>(
        crate_id: CrateId<'a>,
        data: &CrateCrawlerData,
        paths: &mut IndexMap<(CrateId<'a>, Id), Vec<ImportPath>>,
    ) {
        let new_paths = data
            .paths
            .iter()
            .map(|(item_id, paths)| {
                // first we need to convert the paths into actual import paths
                // we need to also calculate visibility
                let import_paths = paths
                    .iter()
                    .map(|p| Self::search_trace_to_import_path(p, &crate_id))
                    .collect::<Vec<_>>();
                ((crate_id, item_id.clone()), import_paths)
            })
            .collect::<IndexMap<_, _>>();
        log::trace!("Paths for crate: {crate_id:?}: {new_paths:#?} ");
        paths.extend(new_paths);
    }

    /// finalizes impls given the globally (across crawled crates) known import paths to ids mapping,
    ///
    /// Returns the external items which were matched up to something in the crate
    fn finalize_impls<'a>(
        crate_id: CrateId<'a>,
        data: &CrateCrawlerData,
        impls: &mut IndexSet<(CrateId<'a>, Id)>,
        ext_traits: &mut IndexMap<(CrateId<'a>, Id), (CrateId<'a>, Id)>,
        crawled_id_map: &gtrie::Trie<char, &(CrateId<'a>, Id)>,
    ) {
        // these paths are often private we need to find them in the crawled map and resolve to a better
        // and most importantly public path, we generate a mapping from the local id's to globally resolved id's
        // anything leftover must be from a crate which was NOT included in the crawl and impls using those need to be filtered out
        for (ext_trait_internal_id, ext_import_path) in &data.external_traits {
            if let Some(ext_trait_global_id) =
                crawled_id_map.get_value(ext_import_path.join("::").chars())
            {
                ext_traits.insert(
                    (crate_id, ext_trait_internal_id.clone()),
                    ext_trait_global_id.clone(),
                );
            }
        }

        let filtered_impls = data.impls.iter().filter_map(|impl_id| {
            data.impls_to_traits
                .get(impl_id)
                .map(|trait_id| {
                    (
                        data.external_traits.contains_key(trait_id),
                        ext_traits.contains_key(&(crate_id, data.impls_to_traits[impl_id].clone())),
                    )
                })
                .and_then(|(is_external, is_resolved)| {
                    (!is_external || is_resolved).then_some((crate_id, impl_id.clone()))
                })
        });

        let filtered_impls = filtered_impls.inspect(|id| {
            log::trace!(
                "Impl `{:?}`, with trait: `{:?}`, trait has path: {} in crate: {}, trait is resolved externally: {}",
                id.1,
                &data.impls_to_traits[&id.1],
                data.paths.contains_key(&data.impls_to_traits[&id.1]),
                crate_id.crate_name(),
                ext_traits.contains_key(&(crate_id, data.impls_to_traits[&id.1].clone())),
            );
        });

        impls.extend(filtered_impls);
    }

    fn finalize_types<'a>(
        crate_id: CrateId<'a>,
        data: &CrateCrawlerData,
        types: &mut IndexSet<(CrateId<'a>, Id)>,
    ) {
        types.extend(std::iter::repeat(crate_id).zip(data.types.iter().cloned()))
    }

    fn search_trace_to_import_path(trace: &[Id], crate_: &Crate) -> ImportPath {
        log::trace!("Converting trace: {trace:?} to import path");
        // paths look like this right now: crate c -> pub module a -> pub use B as C -> struct B = c::a::C
        // imports can rename modules and types
        // IF you can import a struct/enum/trait and it's public, it's public
        let items = trace
            .iter()
            .map(|id| Self::get_item(id, crate_))
            .map(|item| {
                let name: &str;
                let mut is_import = false;
                let is_public = matches!(item.visibility, Visibility::Public);
                let mut is_glob_import = false;
                match &item.inner {
                    ItemEnum::Import(Import {
                        name: imported_name,
                        glob,
                        ..
                    }) => {
                        name = imported_name;
                        is_import = true;
                        is_glob_import = *glob;
                    }
                    _ => name = item.name.as_ref().expect("Expected named item"),
                };
                (name, is_import, is_glob_import, is_public)
            })
            .collect::<Vec<_>>()
            .into_iter();
        let items_prev = once(None).chain(items.clone().map(Option::Some));
        items_prev.zip(items).fold(
            ImportPath::new_public(vec![]),
            |mut import_path, (previous, (current_name, is_import, is_glob, mut is_public))| {
                let mut name = current_name;

                let mut dont_emit_name = is_import;
                if let Some((prev_name, prev_is_import, prev_is_glob, prev_is_public)) = previous {
                    if prev_is_import {
                        is_public |= prev_is_public;
                        name = prev_name;
                    }
                    // glob imports implies this is a module, which we don't want to be part of the path
                    if prev_is_glob {
                        dont_emit_name = true;
                    }
                };

                if !dont_emit_name {
                    import_path.components.push(name.to_owned());
                }

                if !is_public {
                    // any part of the import path being private makes the whole thing private
                    import_path.is_public = false;
                }
                log::trace!(
                    "item - name: `{name}`, is_public: `{is_public}`, is_import: `{is_import}`, is_glob: `{is_glob}` import path is now: `{import_path}`, public?: {}", import_path.is_public
                );
                import_path
            },
        )
    }

    /// get item and panic if not found
    fn get_item<'b>(id: &Id, crate_: &'b Crate) -> &'b Item {
        crate_.index
            .get(id)
            .unwrap_or_else(|| panic!("Expected to find item with id: `{id:?}` but it was not present in the given crate index"))
    }

    fn try_get_item<'b>(id: &Id, crate_: &'b Crate) -> Option<&'b Item> {
        crate_.index.get(id)
    }

    pub fn crawl_crate(&mut self, crate_: &Crate) {
        log::trace!(
            "Crawling crate: `{}` with id: `{:?}`",
            crate_name(crate_),
            crate_.root
        );
        let crate_name = crate_name(crate_);
        self.data.insert(crate_name.to_owned(), Default::default());
        self.crawl_item(crate_.root.clone(), Borrowed(&[]), crate_, crate_name)
    }

    /// Perform depth-first search starting from this item, keep track of the generated path
    /// so that we don't have to re-lookup crate sources later, avoid recursion by keeping track of
    /// visited id's on the way.
    #[allow(clippy::too_many_arguments)]
    fn crawl_item(&mut self, id: Id, mut path: Cow<[Id]>, crate_: &Crate, crate_name: &str) {
        let item = match Self::try_get_item(&id, crate_) {
            Some(v) => v,
            None => {
                log::trace!("Could not find item in index. skipping.");
                return;
            }
        };

        log::trace!(
            "Found matching item in index: `{:?}`, item is a/an: `{}`",
            item.id,
            print_item_variant(&item.inner)
        );

        let children;

        // do we save the current path + emit if any for this item
        let mut register_path_for_item = false;

        match &item.inner {
            ItemEnum::Module(mod_) => {
                children = mod_.items.to_owned();
            }
            ItemEnum::Import(import) => {
                children = import
                    .id
                    .as_ref()
                    .map(|id| vec![id.clone()])
                    .unwrap_or_default();
            }
            ItemEnum::Trait(Trait {
                implementations, ..
            }) => {
                children = implementations.clone();
                register_path_for_item = true;
            }
            ItemEnum::Enum(Enum { impls, .. }) | ItemEnum::Struct(Struct { impls, .. }) => {
                self.data[crate_name].types.insert(id.clone());
                children = impls.to_owned();
                register_path_for_item = true;
            }
            ItemEnum::Impl(Impl { trait_, for_, .. }) => {
                // keep track of impls
                if let Some(trait_) = trait_ {
                    if !crate_.index.contains_key(&trait_.id) {
                        if !crate_.paths.contains_key(&trait_.id) {
                            log::trace!("Impl is for an external trait: `{:?}` which does not exist in the index or external paths, excluding the impl as cannot find import path for trait", trait_.id);
                            return;
                        }
                        // if the trait is external, we won't encounter it on the normal path,
                        // we have to add the import path from here, this might add traits which are
                        // not visible from the public API (i.e. )
                        log::trace!("Impl is for external trait: `{:?}`, we won't find it in the index, saving import path from .paths", trait_.id);
                        self.data[crate_name]
                            .external_traits
                            .insert(trait_.id.clone(), crate_.paths[&trait_.id].path.to_owned());
                    }

                    // store impl to trait mapping
                    if let Some(existing) = self.data[crate_name]
                        .impls_to_traits
                        .insert(id.clone(), trait_.id.clone())
                    {
                        if existing != trait_.id {
                            panic!(
                            "Impl already has a trait mapping: impl: `{id:?}`, for trait: `{existing:?}` but tried replacing with: `{:?}`",trait_.id
                        );
                        }
                    }
                };
                self.data[crate_name].impls.insert(id.clone());
                // impls don't have import paths, so simply return
                log::trace!("Item is an Impl: `{id:?}` for trait: `{:?}` for type: {for_:?}, so skipping but keeping track of Id", trait_.as_ref().map(|t| &t.id) );
                return;
            }
            // we are not interested in these
            _ => {
                log::trace!("Item is being skipped as it is not any of the types we're interested in: `{:?}`, name: {:?}",id, item.name);
                return;
            }
        };
        log::trace!("Item name: `{:?}`, children: `{:?}`", item.name, children);

        path.to_mut().push(id.clone());
        // keep track of this item now
        if register_path_for_item {
            self.store_path(&id, &path, crate_name)
        }

        for (idx, child) in children.into_iter().enumerate() {
            log::trace!("Child no: {idx} from parent id: {id:?}");
            if path.contains(&child) {
                log::trace!("Skipping child as it's already in the path.");
                continue;
            }
            self.crawl_item(child, path.clone(), crate_, crate_name);
        }
    }

    /// Either ignore the given path for this item or write it if it's new or shorter than the existing record
    fn store_path(&mut self, id: &Id, path: &[Id], crate_name: &str) {
        log::trace!("Storing path for id: `{id:?}` in crate: `{crate_name}`: {path:?}");
        // if a path was already found, use the shorter of the two
        self.data[crate_name]
            .paths
            .entry(id.to_owned())
            .or_default()
            .push(path.to_vec());
    }
}

/// Get the name of this crate
pub fn crate_name(crate_: &Crate) -> &str {
    crate_
        .index
        .get(&crate_.root)
        .as_ref()
        .unwrap()
        .name
        .as_ref()
        .unwrap()
}

/// out of the given crates figure out which one the given item belongs to if any
pub fn lookup_item_crate_source<'a>(id: &Id, crates: &'a [Crate]) -> Option<&'a Crate> {
    crates.iter().find(|crate_| crate_.index.contains_key(id))
}

// pub fn get_path(id: &Id, source: &Crate) -> Option<Vec<Id>> {
//     log::debug!(
//         "Trying to find path for item id: `{id:?}` has index entry: `{}` has path entry: `{}`",
//         source.index.get(id).is_some(),
//         source.paths.get(id).is_some()
//     );
//     if source.index.get(id).is_none() {
//         panic!("Trying to find path for item which is external to the provided source crate, the item lives in crate: `{}` not in `{}`",
//             source.external_crates.get(&source.paths.get(id).as_ref().unwrap().crate_id).unwrap().name,
//             crate_name(source)
//         );
//     }
//     match source.paths.get(id) {
//         Some(_) => return Some(vec![id.to_owned()]),
//         None => {
//             let ind = source.index.get(id)?;
//             if let Visibility::Restricted { parent, .. } = &ind.visibility {
//                 if let Some(p_path) = get_path(parent, source) {
//                     return Some(p_path);
//                 }
//             }
//             let parents = source.index.iter().filter(|(_, p_item)| {
//                 // if let Some(name) = &ind.name {
//                 //     if p_item.links.contains_key(name) {
//                 //         log::debug!(
//                 //             "parent via item.links in: `{:?}` named: `{:?}`",
//                 //             p_item.id,
//                 //             p_item.name
//                 //         );

//                 //         return true;
//                 //     }
//                 // }
//                 // if let ItemEnum::Impl(p_impl) = &p_item.inner {
//                 //     return p_impl.items.contains(id);
//                 // }
//                 if let ItemEnum::Import(p_import) = &p_item.inner {
//                     if let Some(p_inner) = &p_import.id {
//                         log::debug!(
//                             "parent import found: `{:?}` named: `{:?}` importing: `{:?}`",
//                             p_item.id,
//                             p_import.source,
//                             p_inner
//                         );
//                         return p_inner == id;
//                     }
//                     return false;
//                 }
//                 if let ItemEnum::Module(p_mod) = &p_item.inner {
//                     log::debug!(
//                         "parent module found: `{:?}` named: `{:?}` containing: `{:?}`",
//                         p_item.id,
//                         p_item.name,
//                         p_mod.items
//                     );
//                     return p_mod.items.contains(id);
//                 }
//                 false
//             });

//             for (parent, _) in parents {
//                 log::debug!("`{id:?}` searching through parent: `{parent:?}`");
//                 let path_o = get_path(parent, source);
//                 if let Some(mut path) = path_o {
//                     log::debug!(
//                         "`{id:?}` found path through parent: `{parent:?}`, path: `{path:#?}`"
//                     );
//                     path.push(id.to_owned());
//                     return Some(path);
//                 }
//             }
//         }
//     };
//     None
// }

// pub fn path_to_import(path: Vec<Id>, source: &Crate) -> Vec<String> {
//     log::debug!(
//         "Trying to convert id path to path components: `{path:?}` with names: [{:?}] in crate: `{}`",
//         path.iter()
//             .map(|id| source
//                 .index
//                 .get(id)
//                 .and_then(|item| item.name.as_deref())
//                 .unwrap_or("None"))
//             .collect::<Vec<_>>()
//             .join(","),
//             crate_name(source)
//     );
//     path.iter()
//         .rev()
//         .enumerate()
//         .rev()
//         .enumerate()
//         .map(|(starti, (endi, id))| {
//             log::trace!("starti: {starti}, endi: {endi}, id: {id:?}");

//             let ind = source
//                 .index
//                 .get(id)
//                 .expect("Trying to find path to item which is not in the provided source crate");

//             if starti == 0 {
//                 return source.paths.get(id).unwrap().path.clone();
//             } else if endi == 0 {
//                 if let Some(name) = &ind.name {
//                     return vec![name.to_owned()];
//                 }
//             } else if let Visibility::Restricted { parent: _, path } = &ind.visibility {
//                 return path[2..].split("::").map(|x| x.to_string()).collect();
//             } else if let ItemEnum::Module(module) = &ind.inner {
//                 if !module.is_stripped {
//                     return vec![source.index.get(id).unwrap().name.clone().unwrap()];
//                 } else {
//                     return vec![];
//                 }
//             }
//             vec![]
//         })
//         .reduce(|mut x, y| {
//             x.extend(y);
//             x
//         })
//         .unwrap()
// }

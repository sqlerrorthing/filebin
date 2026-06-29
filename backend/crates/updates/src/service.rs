pub mod basic;
pub mod rabbitmq;

use std::fmt::Debug;
use domain::persistance::{files, folders};
use futures::Stream;
use service::service;
use std::sync::Arc;
use derive_new::new;
use serde::{Deserialize, Serialize};
use pastey::paste;

macro_rules! make_updates_service {
    (
        $(
            $(#[$meta:meta])*
            $update:ident { $($field:ident : $field_ty:ty),* $(,)? }
            $( | { $($extra_field:ident : $extra_ty:ty),* $(,)? } )?
        );* $(;)?
    ) => {
        #[allow(clippy::redundant_field_names)]
        #[derive(Debug, Clone, Serialize, Deserialize, new)]
        pub struct FolderUpdate {
            pub folder_id: folders::Id,
            pub kind: FolderUpdateKind
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum FolderUpdateKind {
            $(
                $(#[$meta])*
                $update { $( $field : $field_ty ),* }
            ),*
        }

        paste! {
            #[service(dynamic)]
            pub trait UpdatesService {
                /// The stream will close when the folder is deleted.
                /// This stream can be dropped to unsubscribe.
                type FoldersUpdateStream: Stream<Item = Arc<FolderUpdate>>;

                fn subscribe_folder(&self, folder_id: folders::Id) -> Self::FoldersUpdateStream;


                $(
                    fn [<fire_ $update:snake>](&self $(, $($extra_field : $extra_ty)* )? $(, $field : $field_ty )*);
                )*
            }
        }
    };
}

make_updates_service! {
    FileUploaded  { file: files::Model };
    FileDeleted   { file: files::Model };

    FolderRenamed { new_folder_name: String } | { folder_id: folders::Id };
    FolderDeleted { folder: folders::Model };
}

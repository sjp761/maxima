#![allow(non_snake_case)]

use derive_getters::Getters;
use serde::Deserialize;

macro_rules! dip_type {
    (
        $(#[$message_attr:meta])*
        $message_name:ident;
        attr {
            $(
                $(#[$attr_field_attr:meta])*
                $attr_field:ident: $attr_field_type:ty
            ),* $(,)?
        },
        data {
            $(
                $(#[$field_attr:meta])*
                $field:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            // Main struct definition
            $(#[$message_attr])*
            #[derive(Default, Debug, Clone, Deserialize, PartialEq, Getters)]
            #[serde(rename_all = "camelCase")]
            pub struct [<DiP $message_name>] {
                $(
                    $(#[$attr_field_attr])*
                    #[serde(rename = "@" $attr_field)]
                    pub [<attr_ $attr_field>]: $attr_field_type,
                )*
                $(
                    $(#[$field_attr])*
                    pub $field: $field_type,
                )*
            }
        }
    }
}

dip_type!(
    Launcher;
    attr {
        uid: String,
    },
    data {
        file_path: String,
        execute_elevated: Option<bool>,
        trial: bool,
    }
);

dip_type!(
    Runtime;
    attr {},
    data {
        launcher: Vec<DiPLauncher>,
    }
);

dip_type!(
    Manifest;
    attr {
        version: String,
    },
    data {
        runtime: DiPRuntime,
    }
);

impl DiPManifest {
    pub fn file_path(&self, trial: bool) -> Option<String> {
        let launcher = self.runtime.launcher.iter().find(|l| l.trial == trial);
        launcher.map(|l| l.file_path.clone())
    }
}

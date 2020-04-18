

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
use syn::{DeriveInput, Data, AttrStyle, Meta};

extern crate inflector;
use inflector::Inflector;

#[macro_use]
extern crate quote;

extern crate smoldb_traits;
use smoldb_traits::*;

#[proc_macro_derive(Smoldb, attributes(index))]
pub fn smoldb(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).unwrap();

    let gen = impl_smoldb(&ast);

    gen
}

fn impl_smoldb(ast: &syn::DeriveInput) -> TokenStream {
    let s_name = &ast.ident;

    let s = match &ast.data {
        Data::Struct(s) => s,
        _ => panic!("#[derive(Smoldb)] is only defined for structs"),
    };

    let mut field_names = vec![];
    let mut index_names = vec![];

    // Extract fields and indicies
    for f in &s.fields {
        // Skip unnamed fields (because we can't really do anything here)
        let name = match &f.ident {
            Some(v) => v.to_string(),
            None => panic!("#[derive(Smoldb)] structs must only contain named fields"),
        };

        field_names.push(name.clone());

        for a in &f.attrs {
            // Skip inner attributes
            if a.style != AttrStyle::Outer {
                continue;
            }

            // Parse out into meta objects
            let meta = a.parse_meta().expect("Error parsing meta attributes");

            // Handle different meta types
            // TODO: type bindings here
            match meta {
                Meta::Path(p) if p.is_ident("index") => {
                    index_names.push(name.clone());
                },
                _ => (),
            }
        }
    }

    if index_names.len() == 0 {
        panic!("#[derive(Smoldb)] requires at least one field to be marked as an #[index]")
    }

    // Build fields for interpolation

    let mut db_fields = String::new();
    let mut db_field_names = String::new();
    let mut db_values = String::new();
    let mut db_update_fields = String::new();

    let index_enum_name = format_ident!("{}Indicies", s_name);
    let mut index_enum_values = vec![];
    let mut index_enum_name_matches = vec![];
    let mut index_fields = vec![];
    let mut index_names_static = vec![];

    let mut n = 1;

    for i in &index_names {
        // TODO: we should be able to map fields to non-string types here?
        db_fields.push_str(&format!("{} VARCHAR NOT NULL, ", i));

        db_field_names.push_str(&format!("{}, ", i));

        db_values.push_str(&format!("?{}, ", n));

        db_update_fields.push_str(&format!("{} = ?{}, ", i, n));

        let index_enum_variant = format_ident!("{}", i.to_title_case() );
        index_enum_values.push( quote!( #index_enum_variant ) );

        let index_name_static = format_ident!("{}_{}", s_name.to_string().to_screaming_snake_case(), i.to_screaming_snake_case() );
        index_names_static.push(quote!( pub const #index_name_static: &str = #i ));
        index_enum_name_matches.push(quote!( Self::#index_enum_variant(_) => #index_name_static ));

        index_fields.push(format_ident!("{}", i ));

        n += 1;
    }

    db_fields.push_str(&format!("{} BLOB NOT NULL", OBJECT_KEY));
    db_field_names.push_str(OBJECT_KEY);
    db_values.push_str(&format!("?{}", n));
    db_update_fields.push_str(&format!("{} = ?{}", OBJECT_KEY, n));

    // Generate outputs
    let output = quote! {
        /// Static index names for referencing
        #(#index_names_static;)*

        /// Indicies available for querying #s_name objects
        #[derive(Clone, PartialEq, Debug)]
        pub enum #index_enum_name {
            #(#index_enum_values(String),)*
        }

        impl #index_enum_name {
            /// Fetch the column name for the index
            fn name(&self) -> &'static str {
                match self {
                    #(#index_enum_name_matches,)*
                }
            }

            /// Fetch the value for the asspciated index
            fn value<'a> (&'a self) -> &'a str {
                match &self {
                    #(Self::#index_enum_values(v) => v,)*
                }
            }
        }

        /// ToSql implementation allows #index_enum_name to be used as parameters to queries
        impl ToSql for #index_enum_name {
            fn to_sql(&self) -> Result<ToSqlOutput, rusqlite::Error> {
                Ok(ToSqlOutput::Borrowed(self.value().into()))
            }
        }

        impl Storable for #s_name {
            type Indicies = #index_enum_name;

            /// Generate create table SQL statement
            fn sql_create(table_name: &str) -> String {
                format!("CREATE TABLE {} ({});", table_name, #db_fields)
            }

            /// Generate insert SQL statement
            fn sql_insert(table_name: &str) -> String {
                format!("INSERT INTO {} ({}) VALUES ({});", table_name, #db_field_names, #db_values)
            }

            /// Generate select statement with the provided indicies
            fn sql_select(table_name: &str, indicies: &[Self::Indicies]) -> String {
                let w: Vec<String> = indicies.iter().map(|i| format!("{} = ?", i.name() ) ).collect();

                if w.len() == 0 {
                    format!("SELECT {} FROM {};", OBJECT_KEY, table_name)
                } else {
                    format!("SELECT {} FROM {} WHERE {};", OBJECT_KEY, table_name, w.join(", "))
                }
            }

             /// Generate select statement with the provided indicies
             fn sql_update(table_name: &str, indicies: &[Self::Indicies]) -> String {
                let w: Vec<String> = indicies.iter().map(|i| format!("{} = ?", i.name() ) ).collect();

                if w.len() == 0 {
                    format!("UPDATE {} SET {};", table_name, #db_update_fields, )
                } else {
                    format!("UPDATE {} SET {} WHERE {};", table_name, #db_update_fields, w.join(", "))
                }
            }

            /// Generate delete statement with the provided indicies
            fn sql_delete(table_name: &str, indicies: &[Self::Indicies]) -> String {
                let w: Vec<String> = indicies.iter().map(|i| format!("{} = ?", i.name() ) ).collect();

                if w.len() == 0 {
                    format!("DELETE FROM {};", table_name)
                } else {
                    format!("DELETE FROM {} WHERE {};", table_name, w.join(", "))
                }
            }

            /// Fetch parameter values from object instance
            fn params<'a>(&'a self) -> Vec<Box<&'a dyn ToSql>> {
                vec![
                    #(Box::new(&self.#index_fields), )*
                ]
            }
        }
    };

    output.into()
}


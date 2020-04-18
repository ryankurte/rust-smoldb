

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
use syn::{DeriveInput, Data, AttrStyle, Field, Type, Meta, Ident};

extern crate inflector;
use inflector::Inflector;

#[macro_use]
extern crate quote;
use quote::ToTokens;

extern crate smoldb_traits;
use smoldb_traits::*;

#[proc_macro_derive(Smoldb, attributes(index))]
pub fn smoldb(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).unwrap();

    let gen = impl_smoldb(&ast);

    gen
}

/// Implement `Smoldb` derive macro
fn impl_smoldb(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let s = match &ast.data {
        Data::Struct(s) => s,
        _ => panic!("#[derive(Smoldb)] is only defined for structs"),
    };

    // Extract index fields
    let index_fields = extract_fields(s);

    // Create enum name for index enumeration
    let index_enum_name = format_ident!("{}Indicies", struct_name);

    // Build index enum
    let index_enum = build_indicies(&index_enum_name, &index_fields);

    // Implement storable on derived type
    let storable_impl = build_storable(struct_name, &index_enum_name, &index_fields);

    // Generate outputs
    let output = quote! {
        #index_enum
        #storable_impl
    };

    output.into()
}



fn field_to_ident(ty: &Type) -> &Ident {
    let segments = match ty {
        Type::Path(p) => &p.path.segments,
        _ => panic!(),
    };

    let ident = match segments.iter().last() {
        Some(s) => &s.ident,
        _ => panic!(),
    };

    ident
}

#[derive(Debug)]
struct IndexField<'a> {
    pub name: String,

    pub field: &'a Field,

    pub ident: &'a Ident,

    pub sql_type: Option<String>,

    pub is_index: bool,
}

fn extract_fields<'a>(s: &'a syn::DataStruct) -> Vec<IndexField<'a>> {
    let mut index_fields = vec![];

    // Extract fields that should be used for indexing
    for field in &s.fields {
        // Skip unnamed fields (because we can't really do anything here)
        let name = match &field.ident {
            Some(v) => v.to_string(),
            None => panic!("#[derive(Smoldb)] structs must only contain named fields"),
        };

        // Fetch field type
        let ident = field_to_ident(&field.ty);

        let mut i = IndexField { name, field, ident, sql_type: None, is_index: false, };

        for a in &field.attrs {
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
                    i.is_index = true;
                },
                _ => (),
            }
        }

        // Push index fields
        if i.is_index {
            index_fields.push(i);
        }
    }

    if index_fields.len() == 0 {
        panic!("#[derive(Smoldb)] requires at least one field to be marked as an #[index]")
    }

    index_fields
}

fn build_indicies<'a>(index_enum_name: &Ident, index_fields: &[IndexField<'a>]) -> impl ToTokens {

    let mut index_enum_definitions = vec![];
    let mut index_enum_name_matches = vec![];
    let mut index_enum_value_matches = vec![];

    for i in index_fields {
        
        let field_name = &i.name;

        // Override basic field types
        let field_type = format_ident!("{}", i.ident.to_string());

        // Enum variant name
        let index_enum_variant = format_ident!("{}", field_name.to_title_case() );

        // Enum variant definition (in enum declaration)
        index_enum_definitions.push(quote!( #index_enum_variant(#field_type) ));

        // Match for getting index names
        index_enum_name_matches.push(quote!( Self::#index_enum_variant(_) => #field_name ));

        // Match for getting index values
        match &i.field.ty {
            Type::Path(p) if p.path.is_ident("String") => {
                // Strings need to be manually cloned
                index_enum_value_matches.push(quote!( Self::#index_enum_variant(v) => ToSqlOutput::Owned(v.clone().into()) ));
            },
            Type::Path(p) if p.path.is_ident("Vec<u8>") => {
                // Strings need to be manually cloned
                index_enum_value_matches.push(quote!( Self::#index_enum_variant(v) => ToSqlOutput::Owned(v.clone().into()) ));
            },
            _ => {
                index_enum_value_matches.push(quote!( Self::#index_enum_variant(v) => ToSqlOutput::Owned((*v).into()) ));
            }
        }
        
    }

    quote! {
        /// Indicies available for querying #s_name objects
        #[derive(Clone, PartialEq, Debug)]
        pub enum #index_enum_name {
            #(#index_enum_definitions,)*
        }

        impl #index_enum_name {
            /// Fetch the column name for the index
            fn name(&self) -> &'static str {
                match self {
                    #(#index_enum_name_matches, )*
                }
            }

            /// Fetch the value for the asspciated index
            fn value<'a> (&'a self) -> ToSqlOutput {
                match &self {
                    #(#index_enum_value_matches, )*
                }
            }
        }

        /// ToSql implementation allows #index_enum_name to be used as parameters to queries
        impl ToSql for #index_enum_name {
            fn to_sql(&self) -> Result<ToSqlOutput, rusqlite::Error> {
                Ok(self.value())
            }
        }
    }
}

fn build_storable<'a>(struct_name: &Ident, index_enum_name: &Ident, index_fields: &[IndexField<'a>]) -> impl ToTokens {

    let mut db_fields = String::new();
    let mut db_field_names = String::new();
    let mut db_field_values = String::new();
    let mut db_field_updates = String::new();

    let mut index_field_params = vec![];

    let mut n = 1;

    for i in index_fields {

        // Database field for SQL create
        // TODO: allow type overrides here
        db_fields.push_str(&format!("{} VARCHAR NOT NULL, ", i.name));

        // Raw database field names
        db_field_names.push_str(&format!("{}, ", i.name));

        // Database field value entries
        db_field_values.push_str(&format!("?{}, ", n));

        // Database field update entries
        db_field_updates.push_str(&format!("{} = ?{}, ", i.name, n));

        // Fields for all possible indicies
        index_field_params.push(format_ident!("{}", i.name ));

        n += 1;
    }

    db_fields.push_str(&format!("{} BLOB NOT NULL", OBJECT_KEY));
    db_field_names.push_str(OBJECT_KEY);
    db_field_values.push_str(&format!("?{}", n));
    db_field_updates.push_str(&format!("{} = ?{}", OBJECT_KEY, n));

    // Build Storable impl for derived type
    quote! {
        /// Macro-derived storable implementation
        impl Storable for #struct_name {
            type Indicies = #index_enum_name;

            /// Generate create table SQL statement
            fn sql_create(table_name: &str) -> String {
                format!("CREATE TABLE {} ({});", table_name, #db_fields)
            }

            /// Generate insert SQL statement
            fn sql_insert(table_name: &str) -> String {
                format!("INSERT INTO {} ({}) VALUES ({});", table_name, #db_field_names, #db_field_values)
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
                    format!("UPDATE {} SET {};", table_name, #db_field_updates, )
                } else {
                    format!("UPDATE {} SET {} WHERE {};", table_name, #db_field_updates, w.join(", "))
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
                    #(Box::new(&self.#index_field_params), )*
                ]
            }
        }
    }
}



extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
use syn::{DeriveInput, Data, Field, Fields, AttrStyle, Meta};

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

fn impl_smoldb(ast: &syn::DeriveInput) -> TokenStream {
    let s_name = &ast.ident;

    let s = match &ast.data {
        Data::Struct(s) => s,
        _ => panic!("#[derive(Smoldb)] is only defined for structs"),
    };

    let mut fields = vec![];
    let mut indicies = vec![];

    let mut n = 0;

    for f in &s.fields {
        println!("Field: {:?}", f.ident);

        let name = match &f.ident {
            Some(v) => v.to_string(),
            None => continue,
        };

        fields.push(name.clone());

        for a in &f.attrs {
            // Skip inner attributes
            if a.style != AttrStyle::Outer {
                continue;
            }

            let meta = a.parse_meta().expect("Error parsing meta attributes");

            println!("Attribute: {:?}", meta);

            match meta {
                Meta::Path(p) if p.is_ident("index") => {
                    indicies.push(name.clone());
                },
                _ => (),
            }
        }
    }

    if indicies.len() == 0 {
        panic!("#[derive(Smoldb)] requires at least one field to be marked as an #[index]")
    }

    let mut db_fields = String::new();
    let mut db_field_names = String::new();
    let mut db_values = String::new();

    let index_enum_name = format_ident!("{}Indicies", s_name);
    let mut index_enum_values = vec![];
    let mut index_enum_name_matches = vec![];
    let mut index_fields = vec![];
    
    let mut n = 1;

    for i in &indicies {
        // TODO: we should be able to map fields here?
        db_fields.push_str(&format!("{} VARCHAR NOT NULL, ", i));

        db_field_names.push_str(&format!("{}, ", i));

        db_values.push_str(&format!("?{}, ", n));

        let index_enum_variant = format_ident!("{}", i.to_title_case() );
        index_enum_values.push( quote!( #index_enum_variant ) );

        index_enum_name_matches.push(quote!( #index_enum_variant => #i ));

        index_fields.push(format_ident!("{}", i ));

        n += 1;
    }

    db_fields.push_str(&format!("{} BLOB NOT NULL", OBJECT_KEY));
    db_field_names.push_str(OBJECT_KEY);
    db_values.push_str(&format!("?{}", n));

    
    let output = quote! {
        /// Indicies available for querying #s_name objects
        pub enum #index_enum_name {
            #(#index_enum_values(String),)*
        }

        impl #index_enum_name {
            fn name(&self) -> &'static str {
                match self {
                    #(#index_enum_name_matches,)*
                }
            }

            fn value<'a> (&'a self) -> &'a str {
                match &self {
                    #(Self::#index_enum_values(v) => v,)*
                }
            }
        }

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

            fn sql_select(table_name: &str, indicies: &[#index_enum_name]) -> String {
                let w: Vec<String> = indicies.iter().map(|i| format!("{} = ?", i.name() ) ).collect();

                if w.len() == 0 {
                    format!("SELECT {} FROM {};", OBJECT_KEY, table_name)
                } else {
                    format!("SELECT {} FROM {} WHERE {};", OBJECT_KEY, table_name, w.join(", "))
                }
            }
        }

        // TODO: can this be split and generic over instances of Storable?
        impl Store<#s_name> for rusqlite::Connection 
        where
            #s_name: Storable + Serialize + DeserializeOwned,
        {
            type Error = rusqlite::Error;
            type Indicies = <#s_name as Storable>::Indicies;

            fn create_table(&self, table_name: &str) -> Result<(), Self::Error> {
                self.execute(&#s_name::sql_create(table_name), params![])?;

                Ok(())
            }

            fn insert(&self, table_name: &str, t: &#s_name) -> Result<(), Self::Error> {
                let encoded = bincode::serialize(t).unwrap();

                println!("e: {:?}", encoded);

                self.execute(&#s_name::sql_insert(table_name), params![
                    #(t.#index_fields, )*
                    encoded
                ])?;

                Ok(())
            }

            fn select(&self, table_name: &str, indicies: &[Self::Indicies]) -> Result<Vec<#s_name>, Self::Error> {
                let mut query = self.prepare(&#s_name::sql_select(table_name, indicies))?;

                let mut rows = query.query(indicies)?;

                let mut res = Vec::new();

                while let Some(r) = rows.next()? {
                    let d: Vec<u8> = r.get(0)?;

                    let o: #s_name = bincode::deserialize(&d).unwrap();

                    res.push(o);
                }
                
                Ok(res)
            }

        }
    };

    output.into()
}

// TODO: split out generated components

fn impl_indicies(ast: &syn::DeriveInput) -> TokenStream {
    unimplemented!();
}

fn impl_storage(ast: &syn::DeriveInput) -> TokenStream {
    unimplemented!();
}

fn impl_store(ast: &syn::DeriveInput) -> TokenStream {
   unimplemented!(); 
}
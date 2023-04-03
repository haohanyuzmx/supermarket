use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(CreateTable, attributes(index))]
pub fn derive_create_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let data = match input.data {
        Data::Struct(data) => data,
        _ => {
            unimplemented!()
        }
    };

    let str_table_name = name.to_string().to_case(Case::Snake);
    let table_name = format_ident!("{}", &str_table_name);
    let create = generate_create(&data, &table_name.to_string()).to_string();
    quote!(
        impl util::rbatis::init::InitTable for #name {
            fn create_table()->(String,String){
                (String::from(#str_table_name),String::from(#create))
            }
        }
    )
    .into()
}

fn generate_create(data: &DataStruct, table_name: &str) -> String {
    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        _ => unimplemented!(),
    };
    let mut result = "".to_string();
    fields.named.iter().for_each(|field| {
        let field_name = field.ident.as_ref().unwrap().to_string();
        if field_name == "id" {
            result = format!("{result} {},", generate_id());
            return;
        }
        let type_path = match &field.ty {
            syn::Type::Path(ty) => ty,
            _ => unimplemented!(),
        };

        let field_type = match type_path.path.get_ident().unwrap().to_string().as_str() {
            "u32" | "u64" | "i32" | "i64" => "int".to_string(),
            "String" => "varchar(255)".to_string(),
            _ => unimplemented!(),
        };
        result = format!("{result} {field_name} {field_type},");
        if field
            .attrs
            .iter()
            .any(|attr| attr.path().get_ident().unwrap().to_string() == "index")
        {
            result = format!("{result} index({field_name}),")
        };
    });
    result = result[0..result.len() - 1].to_string();
    format!("create table {table_name} ( {result} )")
}

fn generate_id() -> String {
    "id int auto_increment primary key".to_string()
}

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use syn::spanned::Spanned;
use syn::{
    AngleBracketedGenericArguments, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    GenericArgument, Lit, LitStr, Meta, MetaNameValue, PathArguments, Result, Type, TypePath,
};


//--------------------------------------

// #[derive(MysqlEntity)]
// #[derive(Debug)]
// #[table="cf_source"]
// pub struct CfSource {
//     #[pk]
//     #[column="id"]
//     pub id: i64,
//     pub name: String,
//
//     #[column="create_time"]
//     pub gmt_create: DateTime<Local>,
// }

// 查找表名
fn find_table_name_from_deriveinput(st: &DeriveInput) -> syn::Result<Option<String>> {
    for attr in st.attrs.iter() {
        if attr.path.is_ident("table") {
            if let Meta::NameValue(MetaNameValue {
                lit: Lit::Str(list_str),
                ..
            }) = attr.parse_meta()?
            {
                return Ok(Some(list_str.value()));
            }
        }
    }
    Ok(None)
}

// 不是pk的表字段
fn get_non_pk_fields(st: &DeriveInput) -> syn::Result<Vec<&Field>> {
    let fields = match st.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(vec![]);
        }
    };
    let mut normal_fields = Vec::new();

    for field in fields.iter() {
        let is_pk = field
            .attrs
            .iter()
            .any(|f| f.path.is_ident("pk") );

        if !is_pk {
            normal_fields.push(field);
        }
    }
    Ok(normal_fields)
}

fn get_all_fields(st: &DeriveInput) -> syn::Result<Vec<&Field>> {
    let fields = match st.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(vec![]);
        }
    };

    Ok(fields.iter().collect())
}

fn find_pk_filed(st: &DeriveInput) -> syn::Result<Option<&Field>> {
    let fields = match st.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(None);
        }
    };

    for field in fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path.is_ident("pk") {
                return Ok(Some(field));
            }
        }
    }

    Ok(None)
}

fn get_column_name(field: &Field) -> syn::Result<Option<String>> {
    // 查找有没有column属性，没有的，用结构体字段名

    for attr in field.attrs.iter() {
        if attr.path.is_ident("column") {
            if let Meta::NameValue(MetaNameValue {
                lit: Lit::Str(list_str),
                ..
            }) = attr.parse_meta()?
            {
                return Ok(Some(list_str.value()));
            }
        }
    }

    match field.ident {
        None => Ok(None),
        Some(ref v) => Ok(Some(v.to_string())),
    }
}

/*
fn find_datetime_fields(st: &syn::DeriveInput) -> syn::Result<Vec<&Ident>> {
    let fields = match st.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(vec![]);
        }
    };

    let mut field_list = Vec::new();

    for field in fields.iter() {
        if let Type::Path(TypePath { ref path, .. }) = field.ty {
            if let Some(seg) = path.segments.first() {
                if seg.ident == "DateTime" {
                    if let Some(ref ident) = field.ident {
                        field_list.push(ident);
                    }
                }
            }
        }
    }

    Ok(field_list)
}
*/

// 生成删除函数
// 获取表名，pk字段名词，类型
//     pub async fn delete_by_id(pool: &Pool<MySql>, id: i64) -> Result<u64, sqlx::Error> {
//         let sql = "delete from be_user where id = ?";
//         let rst = sqlx::query(sql).bind(id).execute(pool).await?;
//         Ok(rst.rows_affected())
//     }
fn generate_delete_function(st: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let table = match find_table_name_from_deriveinput(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find table attr"));
        }
        Some(v) => v,
    };

    let pk_field = match find_pk_filed(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find pk attr"));
        }
        Some(v) => v,
    };

    let pk_ident = pk_field.ident.clone().unwrap();
    let ty = &pk_field.ty;

    let column = match get_column_name(pk_field)? {
        None => pk_ident.to_string(),
        Some(v) => v,
    };

    let sql_str = format!("delete from {} where {} = ?", table, column);

    let sql_lit = syn::LitStr::new(sql_str.as_str(), st.span());

    let piece = quote::quote! {
         pub async fn delete(#pk_ident: #ty,pool: &sqlx::Pool<sqlx::MySql>) -> std::result::Result<u64, sqlx::Error> {
            let sql = #sql_lit;
            let rst = sqlx::query(sql).bind(#pk_ident).execute(pool).await?;
            Ok(rst.rows_affected())
         }
    };
    Ok(piece)
}

// pub async fn get_by_id(
//         pool: &Pool<MySql>,
//         tz: &FixedOffset,
//         id: i64,
//     ) -> Result<Option<Self>, sqlx::Error> {
//         let sql = "select * from be_user where id = ?";
//         let mut rst = sqlx::query_as::<_, BeUser>(sql)
//             .bind(id)
//             .fetch_optional(pool)
//             .await?;
//
//         if let Some(ref mut v) = rst {
//             mysql_util::fix_read_dt_option(&mut v.last_login, tz);
//             mysql_util::fix_read_dt_option(&mut v.token_expire, tz);
//             mysql_util::fix_read_dt(&mut v.gmt_create, tz);
//             mysql_util::fix_read_dt(&mut v.gmt_modified, tz);
//         }
//
//         Ok(rst)
//     }

fn generate_select_date_piece(st: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let fields = get_all_fields(st)?;
    let mut piece_list = Vec::new();

    for field in fields.iter() {
        let ident = field.ident.clone().unwrap();
        match is_datetime_field(field)? {
            1 => {
                piece_list.push(quote::quote! {
                    mysql_util::fix_read_dt(&mut v.#ident, tz);
                });
            }
            2 => {
                piece_list.push(quote::quote! {
                    mysql_util::fix_read_dt_option(&mut v.#ident, tz);
                });
            }
            _ => {}
        }
    }

    if piece_list.is_empty() {
        Ok(TokenStream2::new())
    } else {
        Ok(quote::quote! {
            if let Some(ref mut v) = rst {
                #(#piece_list);*
            }
        })
    }
}

fn generate_select_function(st: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let table = match find_table_name_from_deriveinput(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find table attr"));
        }
        Some(v) => v,
    };

    let pk_field = match find_pk_filed(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find pk attr"));
        }
        Some(v) => v,
    };
    let ident = &st.ident;
    let pk_ident = pk_field.ident.clone().unwrap();
    let ty = &pk_field.ty;

    let column = match get_column_name(pk_field)? {
        None => pk_ident.to_string(),
        Some(v) => v,
    };

    let sql_str = format!("select * from {} where {} = ?", table, column);
    let sql_lit = syn::LitStr::new(sql_str.as_str(), st.span());

    let sql_piece = quote::quote! {
        let sql = #sql_lit;
    };

    let date_piece = generate_select_date_piece(st)?;

    let piece = quote::quote! {
        pub async fn load(
            #pk_ident: #ty,
            pool: &sqlx::Pool<sqlx::MySql>,
            tz: &chrono::FixedOffset
        ) -> std::result::Result<Option<Self>, sqlx::Error> {
            #sql_piece
            let mut rst = sqlx::query_as::<_, #ident>(sql)
                .bind(#pk_ident)
                .fetch_optional(pool)
                .await?;

            #date_piece
            Ok(rst)
        }
    };
    Ok(piece)
}

// 判断是否是 DateTime或 Option<DateTime>
// 1: DateTime 2:Option<DateTime> 0: non Datetime
// 例如：std::option::Option<chrono::DateTime<chrono::Local>>,
fn is_datetime_field(field: &Field) -> Result<u8> {
    let ty = match field.ty {
        Type::Path(ref v) => v,
        _ => {
            return Err(syn::Error::new_spanned(field, "not a TypePath"));
        }
    };

    // 检查DateTime类型
    let is_datetime = ty.path.segments.iter().any(|f| f.ident == "DateTime");
    if is_datetime {
        return Ok(1);
    }

    // 检查 Option<DateTime>类型
    let option_seg = match ty.path.segments.iter().find(|f| f.ident == "Option") {
        None => {
            return Ok(0);
        }
        Some(v) => v,
    };
    if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { ref args, .. }) =
        option_seg.arguments
    {
        // 查找有没有 DateTime
        if let Some(GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.first() {
            if path.segments.iter().any(|f| f.ident == "DateTime") {
                return Ok(2);
            }
        }
    }

    Ok(0)
}

// 生成 MySqlArguments
/*
fn generate_insert_arguments(st: &syn::DeriveInput) -> Result<Vec<TokenStream2>> {
    let pieces = Vec::new();

    let fields = match st.data {
        Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return Err(syn::Error::new_spanned(
                &st,
                "can't find Punctuated".to_string(),
            ));
        }
    };

    for field in fields.iter() {
        let is_pk = field
            .attrs
            .iter()
            .any(|f| if f.path.is_ident("pk") { true } else { false });
        if is_pk {
            // 主键的不添加
            continue;
        }

        if let Type::Path(TypePath { ref path, .. }) = field.ty {
            if let Some(seg) = path.segments.first() {
                // 判断是否 DateTime字段

                if seg.ident == "DateTime" {
                    if let Some(ref ident) = field.ident {}
                }
            }
        }
    }

    Ok(pieces)
}
*/

// pub async fn insert(&self, pool: &Pool<MySql>, tz: &FixedOffset) -> Result<u64, sqlx::Error> {
//         let sql = "insert into be_user(name,login_name,password,salt, token,phone,email,service_flag,ref_count,last_login,token_expire,memo,gmt_create,gmt_modified) values(?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
//         let mut args = MySqlArguments::default();
//
//         args.add(self.name.clone());
//         args.add(self.login_name.clone());
//         args.add(self.password.clone());
//         args.add(self.salt.clone());
//
//         args.add(self.token.clone());
//         args.add(self.phone.clone());
//         args.add(self.email.clone());
//         args.add(self.service_flag.clone());
//         args.add(self.ref_count.clone());
//
//         args.add(mysql_util::fix_write_dt_option(&self.last_login, tz));
//         args.add(mysql_util::fix_write_dt_option(&self.token_expire, tz));
//         args.add(self.memo.clone());
//         args.add(mysql_util::fix_write_dt(&self.gmt_create, tz));
//         args.add(mysql_util::fix_write_dt(&self.gmt_modified, tz));
//
//         let rst = sqlx::query_with(sql, args).execute(pool).await?;
//         Ok(rst.last_insert_id())
//     }

fn generate_insert_function(st: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let table = match find_table_name_from_deriveinput(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find table attr"));
        }
        Some(v) => v,
    };

    let nonpk_fields = get_non_pk_fields(st)?;
    if nonpk_fields.is_empty() {
        return Err(syn::Error::new_spanned(st, "non-pk fields is empty"));
    }

    let mut nonpk_columns = Vec::new();
    for field in nonpk_fields.iter() {
        let column = get_column_name(field)?.unwrap();
        nonpk_columns.push(column);
    }

    let columns_str = nonpk_columns.join(",");
    let mut question_marks = "?,".repeat(nonpk_columns.len());
    let _ = question_marks.split_off(question_marks.len() - 1);
    let sql_str = format!(
        "insert into {}({}) values({})",
        table, columns_str, question_marks
    );
    let sql_lit = LitStr::new(sql_str.as_str(), st.span());

    let mut mysql_arguments_piece = Vec::new();
    for field in nonpk_fields.iter() {
        let ident = &field.ident.clone().unwrap();
        let argument_piece = match is_datetime_field(field)? {
            1 => {
                quote::quote! {
                    args.add(mysql_util::fix_write_dt(&self.#ident, tz));
                }
            }
            2 => {
                quote::quote! {
                    args.add(mysql_util::fix_write_dt_option(&self.#ident, tz));
                }
            }
            _ => {
                quote::quote! {
                    args.add(self.#ident.clone());
                }
            }
        };
        mysql_arguments_piece.push(argument_piece);
    }

    let piece = quote::quote! {

         pub async fn insert(&self, pool: &sqlx::Pool<sqlx::MySql>,
                        tz: &chrono::FixedOffset,) -> Result<u64, sqlx::Error> {
            let sql = #sql_lit;
            let mut args = sqlx::mysql::MySqlArguments::default();

            #(#mysql_arguments_piece);*

            let rst = sqlx::query_with(sql, args).execute(pool).await?;
            Ok(rst.last_insert_id())
        }


    };
    Ok(piece)
}

// pub async fn update(&self, pool: &Pool<MySql>, tz: &FixedOffset) -> Result<u64, sqlx::Error> {
//         let sql = "update be_user set name = ?, ... ,gmt_modified =? where id = ?";
//         let mut args = MySqlArguments::default();
//
//         args.add(self.name.clone());
//         args.add(self.login_name.clone());
//         args.add(self.password.clone());
//         args.add(self.salt.clone());
//
//         args.add(self.token.clone());
//         args.add(self.phone.clone());
//         args.add(self.email.clone());
//         args.add(self.service_flag.clone());
//         args.add(self.ref_count.clone());
//
//         args.add(mysql_util::fix_write_dt_option(&self.last_login, tz));
//         args.add(mysql_util::fix_write_dt_option(&self.token_expire, tz));
//         args.add(self.memo.clone());
//         args.add(mysql_util::fix_write_dt(&self.gmt_create, tz));
//         args.add(mysql_util::fix_write_dt(&self.gmt_modified, tz));
//
//         let rst = sqlx::query_with(sql, args).execute(pool).await?;
//         Ok(rst.last_insert_id())
//     }

fn generate_update_function(st: &syn::DeriveInput) -> syn::Result<TokenStream2> {
    let table = match find_table_name_from_deriveinput(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find table attr"));
        }
        Some(v) => v,
    };

    let pk_field = match find_pk_filed(st)? {
        None => {
            return Err(syn::Error::new_spanned(st, "can't find pk attr"));
        }
        Some(v) => v,
    };
    let pk_ident = pk_field.ident.clone().unwrap();
    let pk_column = match get_column_name(pk_field)? {
        None => pk_ident.to_string(),
        Some(v) => v,
    };

    let nonpk_fields = get_non_pk_fields(st)?;
    if nonpk_fields.is_empty() {
        return Err(syn::Error::new_spanned(st, "non-pk fields is empty"));
    }

    let mut nonpk_columns = Vec::new();
    for field in nonpk_fields.iter() {
        let column = get_column_name(field)?.unwrap();
        nonpk_columns.push(column);
    }

    let columns_str = nonpk_columns.iter().map(|f| format!("{} = ?",f))
                            .collect::<Vec<String>>().join(",");
    let mut question_marks = "?,".repeat(nonpk_columns.len());
    let _ = question_marks.split_off(question_marks.len() - 1);
    let sql_str = format!(
        "update {} set {} where {} = ?",
        table, columns_str, pk_column
    );
    let sql_lit = LitStr::new(sql_str.as_str(), st.span());

    let mut mysql_arguments_piece = Vec::new();
    for field in nonpk_fields.iter() {
        let ident = &field.ident.clone().unwrap();
        let argument_piece = match is_datetime_field(field)? {
            1 => {
                quote::quote! {
                    args.add(mysql_util::fix_write_dt(&self.#ident, tz));
                }
            }
            2 => {
                quote::quote! {
                    args.add(mysql_util::fix_write_dt_option(&self.#ident, tz));
                }
            }
            _ => {
                quote::quote! {
                    args.add(self.#ident.clone());
                }
            }
        };
        mysql_arguments_piece.push(argument_piece);
    }
    mysql_arguments_piece.push(quote::quote! {
        args.add(self.#pk_ident.clone());
    });

    let piece = quote::quote! {

         pub async fn update(&self, pool: &sqlx::Pool<sqlx::MySql>,
                        tz: &chrono::FixedOffset,) -> Result<u64, sqlx::Error> {
            let sql = #sql_lit;
            let mut args = sqlx::mysql::MySqlArguments::default();

            #(#mysql_arguments_piece);*

            let rst = sqlx::query_with(sql, args).execute(pool).await?;
             Ok(rst.rows_affected())
        }

    };
    Ok(piece)
}


/*
fn get_fields_from_derive_input(st: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = st.data
    {
        Ok(named)
    } else {
        Err(syn::Error::new_spanned(
            &st,
            "can't find Punctuated".to_string(),
        ))
    }
}
*/

/*
fn travel_it(st: &syn::DeriveInput) {
    eprintln!("==== struct attr =======");
    for attr in st.attrs.iter() {
        eprintln!("{:#?}", attr);
    }

    if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = &st.data
    {
        eprintln!("{:#?}", fields.named.len());

        for field in fields.named.iter() {
            eprintln!("---------------------");

            eprintln!("{:#?}", field.attrs);
        }
    }
}
*/

#[proc_macro_derive(MysqlEntity, attributes(table, pk, column))]
pub fn mysql_entity(input: TokenStream) -> TokenStream {
    let ast = syn::parse::<DeriveInput>(input).unwrap();

    let piece_delete_function = generate_delete_function(&ast).unwrap();
    let piece_select_by_id = generate_select_function(&ast).unwrap();
    let piece_insert_function = generate_insert_function(&ast).unwrap();
    let piece_update_function = generate_update_function(&ast).unwrap();

    let ident = &ast.ident;

    let piece = quote::quote! {
        impl #ident {
            #piece_select_by_id
            #piece_insert_function
            #piece_update_function
            #piece_delete_function
        }
    };
    piece.into()
}

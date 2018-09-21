#[macro_use]
extern crate nom;
extern crate rson_rs;
extern crate clap;
extern crate appgen;

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use std::str::{FromStr, from_utf8};    
use std::ops::{RangeFrom, RangeTo, Range};
use nom::*;
use nom::types::CompleteStr;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use appgen::schema_spec::*;
use appgen::sql_gen::*;
use appgen::view_gen::*;
use clap::{Arg, App};

#[macro_use]
extern crate serde_derive;

use rson_rs::de::{from_str, Error};

use std::io::BufWriter;
use std::fs::DirBuilder;

fn build_sql(root: &PathBuf, schema: &SchemaSpec) {
    let mut me = root.clone();
    me.push("sql");
    DirBuilder::new().recursive(true).create(me).unwrap();

    me = root.clone();
    me.push("sql");
    me.push("schema.sql");

    let mut f = File::create(me).unwrap();
    let mut wr = BufWriter::new(f);

    let _:Vec<()> = schema.tables.iter().map(|v| {
        write!(wr, "{}\n", String::from(gen_create_sql(v)));
    }).collect();

    let _:Vec<()> = schema.relationships.iter().map(|v| {
        write!(wr, "{}\n", gen_create_sql(v));
    }).collect();
}

fn make_admin_tag(rsrc: &str, views: &Vec<(String, String)>) -> JsxNode {
    // TODO(matt) - Label
    let mut hm = HashMap::new();
    hm.insert("name".to_string(), JsxAttr::JsxString(rsrc.into()));
            // "title".to_string() => rsrc.label.as_str().into()
            //);

    for i in views {
        hm.insert(i.0.clone(), JsxAttr::JsxVariableRef(i.1.clone()));
    }
    JsxNode::Leaf("Resource".to_string(), Some(hm))
}



fn write_jsx(root: &PathBuf, schema: &SchemaSpec, dpdef: (&str, &str)) {
    let mut me = root.clone();
    me.push("web");
    me.push("src");
    DirBuilder::new().recursive(true).create(me).unwrap();


    let mut resource_views:HashMap<String, Vec<(String, String)>> = HashMap::new();


    let global_imports:Vec<Import> = schema.views
        .iter()
        .map(|v| {
            let (view_name, node, import) = top_level_view_node(&schema.tables, v);

            let mut me = root.clone();
            me.push("web");
            me.push("src");
            me.push(view_name.clone()+ ".js");

            resource_views
                .entry(v.source.name.clone())
                .or_insert_with(|| vec!())
                .push((v.view.view_attr_name(), view_name.clone()));


            let mut f = File::create(me).unwrap();
            let mut wr = BufWriter::new(f);


            write!(wr,
                    "{}\n\nexport const {} = (props) => (\n{}\n);\n", 
                    import.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("\n"),
                    view_name,
                    node.to_string(true, "".into()));
            let source = String::from("./") + &view_name;
            Import::new(&source, vec!(&view_name))
        }).collect();

    // Theres probably a better way to do this
    let mut me = root.clone();
    me.push("web");
    me.push("src");
    me.push("App.js");
    
    let mut f = File::create(me).unwrap();
    let mut wr = BufWriter::new(f);



    // TODO(matt) - refactor to function




    write!(wr, "import React from 'react';\n");
    write!(wr, "import {{ Admin, Resource }} from 'react-admin';\n");
    write!(wr, "{};\n", dpdef.1);

    let _: Vec<()> = global_imports.iter().map(|i| {
        write!(wr, "{}\n", i.to_string());
    }).collect();
     

    let admin_children: Vec<JsxNode> = resource_views.iter()
        .map(|(r, vs)| {
            make_admin_tag(r, vs)
        }).collect();

    let mut hm:HashMap<String, JsxAttr> = HashMap::new();
    hm.insert("dataProvider".into(), JsxAttr::JsxVariableRef(dpdef.0.into()));

    write!(wr, "const App = () => (\n{}\n);\nexport default App;",
        JsxNode::node("Admin", Some(hm), admin_children).to_string(true, "".into()));
}



fn read_appspec(filename: &str) -> SchemaSpec {
    let mut contents = String::new();
    let mut f = File::open(filename).expect("file not found");
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let t: Result<SchemaSpec, rson_rs::de::Error> = from_str(&contents);
    t.unwrap()
}


fn main() {
    let matches = App::new("App Gen")
        .version("0.1")
        .author("Matt Woodyard <matt@mattwoodyard.com>")
        .about("Generate an entire web app from a description of the data types")
        .arg(Arg::with_name("INPUT")
                .value_name("APPSPEC_FILE")
                .help("Read the application specification from a file")
                .takes_value(true)
                .index(1)
                .required(true))
        .arg(Arg::with_name("OUTPUT")
                .value_name("OUTPUT_DIRECTORY")
                .help("Directory for output")
                .takes_value(true)
                .index(2)
                .required(true))
        .arg(Arg::with_name("DATA_PROVIDER_CONSTRUCTOR")
                .value_name("DATA_PROVIDER_CONSTRUCTOR")
                .short("p")
                .help("Construction statement for data provider")
                .takes_value(true)
                .required(false))
        .arg(Arg::with_name("DATA_PROVIDER_IMPORT")
                .value_name("DATA_PROVIDER_IMPORT")
                .short("i")
                .help("Import statement for data provider")
                .takes_value(true)
                .required(false))
        .get_matches();

    let dp = (matches.value_of("DATA_PROVIDER_CONSTRUCTOR").unwrap_or("make thing"),
            matches.value_of("DATA_PROVIDER_IMPORT").unwrap_or("import thing"));


    let filenames = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    let schema = read_appspec(filenames);

    let target_dir = PathBuf::from(output_path);

    build_sql(&target_dir, &schema);
    write_jsx(&target_dir, &schema, dp);
}





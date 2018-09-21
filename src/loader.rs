
extern crate toml;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::env;
use std::io;
use std::io::prelude::*;

use toml::Value as Toml;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;


#[derive(Deserialize, Serialize, Debug)]
struct TableDef {
    name: String
}
#[derive(Deserialize, Serialize, Debug)]
enum ViewVisible {
    Hidden,
    Default,
    Visible
}

#[derive(Deserialize, Serialize, Debug)]
struct ColumnDef {
    table: String,
    name: String,
    label: Option<String>,
    widgets: Option<HashMap<String, String>>,

    #[serde(rename="type")]
    column_type: String,

    type_parameters: HashMap<String, String>,
    ui_view: Option<HashMap<String, ViewVisible>>
}

#[derive(Deserialize, Serialize, Debug)]
struct AclDef {
    role: Option<String>,
    resource: Option<String>,
    permission: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
struct IndexDef {
    table: String,
    columns: Vec<String>,

    #[serde(rename = "primary-key")]
    primary_key: Option<Vec<String>>

}

struct ConstraintDef {
}


#[derive(Deserialize, Serialize, Debug)]
enum LocationDef {
    Backend,
    Endpoint,
    Frontend
}



#[derive(Deserialize, Serialize, Debug)]
struct TriggerDef {
    event_name: String,
    event_location: LocationDef,
    source: String,
}

/* enum TriggerDef { */
/*     Backend { name: String, source: String }, */ 
/*     Endpoint { url: String }, */
/*     Frontend { function_name: String, source: String } */
/* } */




#[derive(Deserialize, Serialize, Debug)]
struct RelationshipDef {
   column: String,
   #[serde(rename ="references")]
   reference_to: String,
}


enum Define {
    Table(Rc<RefCell<TableDef>>),
    Column(Rc<RefCell<ColumnDef>>),
    Acl(Rc<RefCell<AclDef>>),
    Index(Rc<RefCell<IndexDef>>),
    Constraint(Rc<RefCell<ConstraintDef>>),
    Trigger(Rc<RefCell<TriggerDef>>),
    Relationship(Rc<RefCell<RelationshipDef>>),
}

#[derive(Deserialize, Serialize, Debug)]
struct SchemaDef {
    #[serde(rename = "define-table")]
    tables: Option<Vec<TableDef>>,

    #[serde(rename = "define-column")]
    columns: Option<Vec<ColumnDef>>,

    #[serde(rename = "define-acl")]
    acls: Option<Vec<AclDef>>,

    #[serde(rename = "define-index")]
    indices: Option<Vec<IndexDef>>,

    #[serde(rename = "define-event")]
    events: Option<Vec<TriggerDef>>,

    #[serde(rename = "define-relationship")]
    relationships: Option<Vec<RelationshipDef>>

}





fn extract(toml: Toml) -> Vec<Define> {
    match toml {
        Toml::Table(table) => {
            println!("Toml: {:?}" , table);

        }
        _ => { println!("----"); }
    }
    vec!()
}




fn main() {
    let mut args = env::args();
    let mut input = String::new();
    if args.len() > 1 {
        let name = args.nth(1).unwrap();
        File::open(&name).and_then(|mut f| {
            f.read_to_string(&mut input)
        }).unwrap();
    } else {
        io::stdin().read_to_string(&mut input).unwrap();
    }

     let cfg: SchemaDef =  toml::from_str(&input).unwrap();
     println!("{:?}", cfg);
    /* let evts = vec!( */
    /*     TriggerDef{ name: String::from("foo"), source: String::from("bar"), location: LocationDef::Backend } */
    /* ); */ 
    /* /1* let cfg = SchemaDef { tables: None, columns: None, acls: None, indices: None, *1/ */ 
    /*     events:Some(evts), */
    /*     relationships: None }; */



}


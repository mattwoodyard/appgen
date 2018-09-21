
extern crate toml;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::env;
#[macro_use]
use std::fmt;
use std::io;
use std::io::prelude::*;

use toml::Value as Toml;
use serde_json::Value as Json;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;



macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $val); )*
            map
    }}
}


#[derive(Deserialize, Serialize, Debug)]
enum Visibility {
    Hidden,
    Default,
    Visible


}





#[derive(Deserialize, Serialize, Debug)]
struct UiDef {
    #[serde(rename = "resource")]
    resources: Vec<ResourceDef>,
    #[serde(rename = "widget")]
    widgets: Vec<ViewWidgetDef>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ViewWidgetDef {
    resource: String,
    field_source: String,
    label: Option<String>,
    widget_type: String,
    order: Option<usize>,
    views: Vec<BaseView>,
    // TODO Parse
    props: Option<HashMap<String, String>>,
    child: Option<Box<ChildViewDef>>,
}



#[derive(Deserialize, Serialize, Debug)]
struct ResourceDef {
    name: String,
    label: String,
    endpoint: Option<String>,
    #[serde(rename = "actions")]
    enabled_actions: Option<Vec<BaseView>>
}



#[derive(Deserialize, Serialize, Debug)]
struct FieldDef {
    source: String, 
    label: Option<String>,
    view: Option<Vec<WidgetDef>>
}



#[derive(Deserialize, Serialize, Debug)]
struct ActionDef { }


#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash)]
enum BaseView {
    Create,
    Default,
    Delete,
    Edit,
    Filter,
    List,
    Show,
}


impl BaseView {

    fn as_key(&self) -> String {
        match self {
            BaseView::Default => { "default".into() }
            BaseView::Create => { "create".into() }
            BaseView::Delete => { "delete".into() }
            BaseView::Edit => { "edit".into() }
            BaseView::Filter => { "filter".into() }
            BaseView::List => { "list".into() }
            BaseView::Show => { "show".into() } 
        } 
    }


}

#[derive(Deserialize, Serialize, Debug)]
enum ChildView {
Array,
    ReferenceArray,
    ReferenceMany,
}



#[derive(Deserialize, Serialize, Debug)]
struct ChildViewDef {
    base_view: ChildView,
    // Boolean
    use_extended: bool,
    #[serde(rename = "widget")]
    widgets: Vec<WidgetDef>,
} 

#[derive(Deserialize, Serialize, Debug)]
struct WidgetDef {
    #[serde(rename = "type")]
    widget_type: String,
    child: Option<Box<ChildViewDef>>,
    props: Option<HashMap<String, String>>
}


/* impl WidgetDef { */ 
/*     fn combine(w1: &Option<WidgetDef>, override: &Option<WidgetDef>) -> WidgetDef {} */
/*     fn to_node(&self) -> Node { */
        
    

/*     } */
/* } */

#[derive(Debug)]
struct Import {
    source: String,
    symbols: Vec<String>
}

impl Import {
    
    fn merge(imports: Vec<Import>) -> Vec<Import> {
        imports
    }
}

impl GenScript for Import {
    fn gen(&self) -> String {
        format!("import {{{}}} from '{}';\n", self.symbols.join(", "), self.source)
    }
}


#[derive(Debug)]
struct ViewsForResource {
    resource_prefix: String,
    imports: Vec<Import>,
    show: Option<Node>,
    list: Option<Node>,
    create: Option<Node>,
    edit: Option<Node>,
    filter: Option<Node>,
}


#[derive(Debug)]
pub enum AttrV {
    JString(String),
    JVarRef(String),
    Object(HashMap<String, String>),
    JObject(HashMap<String, Box<AttrV>>),
    /* JArray(Vec<Box<AttrV>>), */
    JRaw(String),
    Bool(bool),
    Ellipsis(String),
}


impl<'a> From<&'a str> for  AttrV {
    fn from(s: &'a str) -> AttrV {
        AttrV::JString(s.into())
    }
}

pub struct Attrs {
    attrs: HashMap<String, AttrV>
}



#[derive(Debug)]
pub enum Node {
    Node(String, Option<HashMap<String, AttrV>>, Vec<Box<Node>>),
    Leaf(String, Option<HashMap<String, AttrV>>),
    Text(String)
}

impl Node {
    fn node(t: &str, a: Option<HashMap<String, AttrV>>, c: Vec<Node>) -> Node {
        Node::Node(t.into(), a, c.into_iter().map(|i| { Box::new(i) }).collect())
    }


    fn get_tag(&self) -> Option<String> {
        match self {
        Node::Node(t, _, _) => Some(t.clone()),
        Node::Leaf(t, _) => Some(t.clone()),
        _ => None
        }
    }

    fn tag(t: &str) -> Node {
        Node::Node(t.into(), None, vec!())
    }

    fn text(t: &str) -> Node {
        Node::Text(t.into())
    }


    fn extend(&mut self, n: Node) {
        match self {
            Node::Node(t, a, v) => {
                v.push(Box::new(n))
            }
            /* Node::Leaf(t, a) => { */
            /*     *self = Node::Node(*t, *a, vec!(Box::new(n))); */
            /* } */
            _ => {}
        }
    }
}

fn capitalize(word: &str) -> String {
    word.chars().enumerate()
        .map(|(i, c)| if i == 0 { c.to_uppercase().to_string() } else { c.to_lowercase().to_string() })
        .collect()
}

fn view_name(rsname: &str, b: &BaseView) -> String {
    let rname = capitalize(rsname);
    match b {
        BaseView::List => {
            rname+"List"
        }
        BaseView::Edit => {
            rname+"Edit"
        }
        BaseView::Show => {
            rname+"Show"
        }
        BaseView::Create => {
            rname+"Create"
        }
        _ => { "".into() }
    }
}


fn make_admin_tag(rsrc: &ResourceDef) -> Node {
    let mut hm = hashmap!( "name".to_string() => rsrc.name.as_str().into(),
            "title".to_string() => rsrc.label.as_str().into()
            );

    for i in rsrc.enabled_actions.as_ref().unwrap_or(&vec!()) {
        hm.insert(i.as_key(), AttrV::JVarRef(view_name(&rsrc.name, i)));
    }
        

    Node::Leaf("Resource".to_string(), Some(hm))
}

fn make_resource_import(rsrc: &ResourceDef) -> Vec<Import> {
    vec!()
}

fn field_node(source: String, label: String, widget: WidgetDef) -> Node {
    Node::tag("TextField")
}


fn filter_for_view<'a>(rsrc: &str, vname: &BaseView, widgets: &'a Vec<ViewWidgetDef>) -> Vec<&'a ViewWidgetDef> {
    widgets.iter().filter(|w| { w.resource == rsrc && w.views.contains(vname) }).collect()
}


fn build_tag_for_widget(w: &ViewWidgetDef) -> Node {
    let mut hm = HashMap::new();
    if w.field_source != "action" {
        hm.insert("label".into(), AttrV::JString(w.label.clone().unwrap_or_else(|| {w.field_source.clone()})));
        hm.insert("source".into(),  AttrV::JString(w.field_source.clone()));
    }
    Node::node(&w.widget_type, Some(hm), vec!()) 
}



fn make_list_view(rsrc: &ResourceDef, mut widgets: Vec<&ViewWidgetDef>) -> (Vec<Import>, Node) {

    widgets.sort_unstable_by(|a, b|  { a.order.unwrap_or(0).cmp(&b.order.unwrap_or(0)) });

    let kids  = make_widgets(widgets);
    let imports = make_imports(&kids);
        
    let hm = hashmap!( "_".into() => AttrV::Ellipsis("props".into()));
    let root = Node::node("List", Some(hm), vec!(Node::node("Datagrid", None, kids)));
    (imports, root)
}

fn make_imports(kids: &Vec<Node>) -> Vec<Import> {
    kids.iter()
        .flat_map(|n| 
            n.get_tag().map(|t| { Import {source: "unk".into(), symbols: vec!(t.clone()) }})
        ).collect()
}

fn make_widgets(widgets: Vec<&ViewWidgetDef>) -> Vec<Node> {
    widgets.iter()
        .map(|w| { build_tag_for_widget(w) })
        .collect()
}


fn make_form_view(rsrc: &ResourceDef, mut widgets: Vec<&ViewWidgetDef>, form: &str ) -> (Vec<Import>, Node) {
    widgets.sort_unstable_by(|a, b|  { a.order.unwrap_or(0).cmp(&b.order.unwrap_or(0)) });
    let kids  = make_widgets(widgets);
    let imports = make_imports(&kids);
    let hm = hashmap!( "_".into() => AttrV::Ellipsis("props".into()));
    let root = Node::node(form, Some(hm), vec!(Node::node("SimpleForm", None, kids)));
    (imports, root)
}


fn make_edit_view(rsrc: &ResourceDef, mut widgets: Vec<&ViewWidgetDef>) -> (Vec<Import>, Node) {
    make_form_view(rsrc, widgets, "Edit")
}


fn make_views(rsrc: &ResourceDef, widgets: &Vec<ViewWidgetDef> ) -> ViewsForResource {
    // TODO(matt) - only generate and push views with some content

    let (i1, ln) = make_list_view(rsrc, filter_for_view(&rsrc.name, &BaseView::List, widgets));
    let (i2, en) = make_edit_view(rsrc, filter_for_view(&rsrc.name, &BaseView::Edit, widgets));
    let (i3, cn) = make_form_view(rsrc, filter_for_view(&rsrc.name, &BaseView::Create, widgets), "Create");
    let (i4, sn) = make_edit_view(rsrc, filter_for_view(&rsrc.name, &BaseView::Show, widgets));

    let mut i = vec!(); 
    i.extend(i1);
    i.extend(i2);
    i.extend(i3);
    i.extend(i4);

    ViewsForResource {resource_prefix: rsrc.name.clone(), imports: i,  show: Some(sn), list: Some(ln), create: Some(cn), edit: Some(en), filter: None}
}



#[derive(Debug)]
struct WebInterface {
    resource_defs: Node,
    resource_views: HashMap<String, ViewsForResource>,
    imports: Vec<Import>
}

impl<'a> From<&'a UiDef> for WebInterface {

    fn from(u: &UiDef) -> WebInterface {
        let hm = hashmap!(
                "dataProvider".into() => AttrV::JVarRef("postgrestClient('http://localhost:3000/api')".into())
                );

        
        let mut rd = Node::node("Admin", Some(hm), vec!());
        let mut views = HashMap::new();

        let _: Vec<()> = u.resources.iter()
            .map(|r| {
                let rv = make_views(r, &u.widgets);
                views.insert(r.name.clone(), rv);
                rd.extend(make_admin_tag(r));
            }).collect();

        WebInterface {
            resource_defs: rd, 
            resource_views: views,
            imports: vec!()
        }
    }
}

trait GenScript {
    fn gen(&self) -> String;
}

impl<A:GenScript> GenScript for Vec<A> {

    fn gen(&self) -> String {
       let o:Vec<String> = self.iter().map(|i| i.gen()).collect();
       o.join("\n") +"\n"
    }
}

impl<A: GenScript> GenScript for Option<A> {
    fn gen(&self) -> String {
        match self {
            Some(i) => i.gen(),
            None => String::from("")
        }
    }
}

impl<A: GenScript> GenScript for Box<A> {
    fn gen(&self) -> String {
        GenScript::gen(self.as_ref())
    }
}

fn gen_jsx_tag_attrs(a: &Option<HashMap<String, AttrV>>) -> String {
    a.as_ref().map(|hm| {
        let line: Vec<String> = hm.iter().map(|(k,v)| {
                match v {
                    AttrV::Bool(tf) => {
                        if *tf {
                            format!("{}", k)
                        } else {
                            "".into()
                        }
                    }
                    AttrV::JString(s) => {
                        format!("{}=\"{}\"", k, s) 
                    }
                    AttrV::Ellipsis(e) => {
                        format!("{{...{}}}", e) 
                    }
                    _ => {
                        format!("{}={{{}}}", k, v.gen()) 
                    }
                }
            }).collect();
        line.join(" ")
    }).unwrap_or("".into())
}


impl<'a> GenScript for &'a AttrV {
    fn gen(&self) -> String {
        GenScript::gen(*self)
    }
}

impl GenScript for AttrV {
    fn gen(&self) -> String {
        match self {
            AttrV::JString(s) => format!("\"{}\"", s), 
            AttrV::JRaw(s) => s.clone(), 
            AttrV::JVarRef(s) => s.clone(), 
            AttrV::Object(hm) => {
                let s: Vec<String> = hm.iter().map(|(k, v)| { format!("{}: \"{}\"", k, v) }).collect();
                s.join(", ")
            }

            AttrV::JObject(hm) => {
                let s: Vec<String> = hm.iter().map(|(k, v)| { format!("{}: {}", k, v.gen()) }).collect();
                s.join(", ")
            }

            AttrV::Bool(v) => {
                if *v { "true".into() }
                else { "false".into() }
            }

            AttrV::Ellipsis(name) => {
                format!("...{}", name)

            }
        }
    }

}


impl GenScript for Node {

    fn gen(&self) -> String {
        match self {
            Node::Node(t, a, v) => {
                let attr = gen_jsx_tag_attrs(a);
                if attr.len() == 0 && v.len() > 0 {
                    format!("<{}>\n{}</{}>", t, v.gen(), t).into()
                } else if attr.len() > 0 && v.len() > 0 {
                    format!("<{} {}>\n{}</{}>", t, attr, v.gen(), t).into()
                } else if attr.len() == 0 && v.len() == 0 {
                    format!("<{}/>", t).into()
                } else {
                    format!("<{} {}/>", t, attr).into()
                }
            }
 
            Node::Leaf(t, a) => {
                format!("<{} {}/>", t, gen_jsx_tag_attrs(a))
            }

            Node::Text(s) => {
                s.clone()
            }
        }
    }
}

fn jsx_def(name: &str, nodes: &Option<Node>, export: bool) -> String {
    let mut x = nodes.as_ref().map(|n| n.gen()).unwrap_or(String::from(""));
    if x.len() > 0 {
        x = String::from("\n") + &x +"\n";
        let exp = if export { "export " } else { "" };
        format!("{}const {} = (props) => ({});",exp, name , x)
    } else {
        "".into()
    }

}


fn dump(w: WebInterface) {
    for i in w.imports.iter() {
        println!("import {{{}}} from '{}';\n", i.symbols.join(", "), i.source);
    }



    println!("const App = () => (\n{}\n);", w.resource_defs.gen());
    for (k, v) in w.resource_views {
        let ln = view_name(&v.resource_prefix, &BaseView::List);
        println!("{}", jsx_def(&ln, &v.list, true));

        let en = view_name(&v.resource_prefix, &BaseView::Edit);
        println!("{}", jsx_def(&en, &v.edit, true));

        let cn = view_name(&v.resource_prefix, &BaseView::Create);
        println!("{}", jsx_def(&cn, &v.create, true));

        let sn = view_name(&v.resource_prefix, &BaseView::Show);
        println!("{}", jsx_def(&sn, &v.show, true));

        let gn = view_name(&v.resource_prefix, &BaseView::Filter);
        println!("{}", jsx_def(&gn, &v.filter, true));
   }


    println!("export default App;\n");
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

     let cfg: UiDef =  toml::from_str(&input).unwrap();
     dump(WebInterface::from(&cfg));

}


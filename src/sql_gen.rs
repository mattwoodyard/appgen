
use schema_spec::*;

//use std::fmt;

use std::convert::From;


pub fn gen_create_sql<'a, A>(a: &'a A) -> String
  where CreateSql: From<&'a A>
{
  String::from(CreateSql::from(a))
}

impl<'a> From<&'a Field> for CreateSql {

    fn from(f: &Field) -> CreateSql {
        format!("{} {}", f.0, gen_create_sql(&f.1)).into()
    }
}




impl<'a> From<&'a MappedFieldType> for CreateSql {
    fn from(f: &MappedFieldType) -> CreateSql {
        match f {
            MappedFieldType::BigSerialPk => {
               "BigSerial Primary Key".into()
            }
            MappedFieldType::String(max_length) => {
                format!("varchar({})", max_length).into()
            }
            MappedFieldType::Integer => {
                format!("BigInt").into()
            }
            MappedFieldType::Numeric => {
                format!("Numeric").into()
            }
            MappedFieldType::Timestamp(timezone) => {
                format!("Timestamp").into()
            }
            MappedFieldType::Boolean => {
                format!("Boolean").into()
            }

        }
    }
}



impl<'a> From<&'a (String, Vec<Field>)> for CreateSql {
    fn from(s: &(String, Vec<Field>)) -> CreateSql {
        let fields = &s.1;
        let sql: String = fields
            .iter()
            .map(|f| {
                gen_create_sql(f)
            })
            .collect::<Vec<String>>()
            .join(",\n  ");
        CreateSql::from(
            format!("CREATE TABLE {} (\n  {}\n);", s.0, sql)
        )
    }
}

// TODO(matt) - refactor this so it can be run 'online' against a 
//              migration
impl <'a> From<&'a Relation> for CreateSql {
    fn from(s: &Relation) -> CreateSql {
       let a1 = format!("ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {}({});", s.from.table(), s.from.field(), s.to.table(), s.to.field());
//       let a2 = format!("ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {}({});", s.to.table(), s.to.field(), s.from.table(), s.from.field());

       CreateSql::from(a1 + "\n")
    }
}










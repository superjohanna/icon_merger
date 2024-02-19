use std::io::{Read, Write};

use xml::EmitterConfig;
use xmltree::{Element, XMLNode};

use csv::CSV;

pub mod csv;

#[allow(dead_code)]
fn main() {
    let mut csv = CSV::new("./file.csv");
    let mut buf = String::new();

    let mut folder_base_file =
        std::fs::File::open("./places/48/folder.svg").expect("Couldn't open folder.svg");
    let mut folder_base_bytes = Vec::<u8>::new();
    folder_base_file
        .read_to_end(&mut folder_base_bytes)
        .expect("Failed to read to end");
    let folder_base = Element::parse(folder_base_bytes.as_slice()).unwrap();

    for batch in csv {
        let s = "./places/48/".to_owned() + &batch.key;

        let mut folder_x_file_new =
            std::fs::File::create(s).unwrap_or_else(|_| panic!("Failed to create: {}", batch.key));

        let mut s = "./places/16/".to_owned() + &batch.key;

        let mut folder_x_file_old =
            std::fs::File::open(s).unwrap_or_else(|_| panic!("Failed to open: {}", batch.key));

        let mut buf = Vec::<u8>::new();

        folder_x_file_old
            .read_to_end(&mut buf)
            .unwrap_or_else(|_| panic!("Failed to read to end: {}", batch.key));

        let mut folder_icon_small = Element::parse(buf.as_slice()).unwrap();

        let mut folder_icon_large = folder_base.clone();

        let mut gs_large = folder_icon_small
            .children
            .iter_mut()
            .filter(|x| x.as_element().is_some_and(|y| y.name == "g"));

        let mut gs_id_icon_large = gs_large.filter(|x| {
            x.as_element().is_some_and(|y| {
                y.attributes.get_key_value("id")
                    == Some((&"id".to_owned(), &"Icon-small".to_owned()))
            })
        });

        let mut gs_id_icon_vec_large = gs_id_icon_large.collect::<Vec<&mut XMLNode>>();

        let mut gs_id_icon_vec_large_owned =
            gs_id_icon_vec_large.iter().map(|x| (**x).clone()).collect();

        let mut gs_base = folder_icon_large
            .children
            .iter_mut()
            .filter(|x| x.as_element().is_some_and(|y| y.name == "g"));

        let mut gs_id_icon_base = gs_base.filter(|x| {
            x.as_element().is_some_and(|y| {
                y.attributes.get_key_value("id") == Some((&"id".to_owned(), &"Icon".to_owned()))
            })
        });

        let mut gs_id_icon_vec_base = gs_id_icon_base.collect::<Vec<&mut XMLNode>>();

        gs_id_icon_vec_base
            .first_mut()
            .unwrap()
            .as_element_mut()
            .unwrap()
            .children
            .append(&mut gs_id_icon_vec_large_owned);

        let mut conf = EmitterConfig::new();

        conf.perform_indent = true;

        folder_icon_large
            .write_with_config(folder_x_file_new, conf)
            .unwrap();

        /* for g in gs_id_icon {
            println!("{:?}", g);
        } */

        // Create symlinks
        for val in batch.values {
            let val = "./places/48/".to_owned() + &val;
            std::os::unix::fs::symlink(batch.key.clone(), val).unwrap();
        }
    }

    pub trait AsElementMut {
        fn as_element_mut(&mut self) -> Option<&mut Element>;
    }

    impl AsElementMut for XMLNode {
        fn as_element_mut(&mut self) -> Option<&mut Element> {
            match self {
                XMLNode::Element(e) => Some(e),
                _ => None,
            }
        }
    }
}

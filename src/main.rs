use std::fs::{create_dir_all, remove_dir_all};
use std::io::stdin;

use heed::EnvOpenOptions;
use milli::update::{IndexDocumentsMethod, UpdateBuilder, UpdateFormat};
use milli::{Error, Index};

fn main() {
    let data = stdin();
    let db_name = "bug.mmdb";
    match remove_dir_all(db_name) {
        Ok(_) => eprintln!("The previous db has been deleted from the filesystem"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("{}", e),
    }
    create_dir_all(db_name).unwrap();

    let mut options = EnvOpenOptions::new();
    options.map_size(100 * 1024 * 1024 * 1024); // 100 GB
    options.max_readers(10);
    let index = Index::new(options, db_name).unwrap();

    let update_builder = UpdateBuilder::new(0);
    let mut wtxn = index.write_txn().unwrap();
    let mut builder = update_builder.index_documents(&mut wtxn, &index);
    builder.update_format(UpdateFormat::Csv);
    builder.index_documents_method(IndexDocumentsMethod::ReplaceDocuments);

    match builder.execute(data, |_, _| ()) {
        Err(Error::UserError(e)) => println!("{:?}", e),
        Err(Error::IoError(e)) => println!("{:?}", e),
        Err(Error::InternalError(e)) => panic!("{:?}", e),
        Ok(_) => (),
    }

    wtxn.commit().unwrap();

    index.prepare_for_closing().wait();

    println!("everything was indexed without issue");
}

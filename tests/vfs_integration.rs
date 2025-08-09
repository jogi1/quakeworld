mod datareader_integration;

use std::default;
use std::fs::File;

use quakeworld::datatypes::common::AsciiString;
use quakeworld::pak::Pak;
use quakeworld::utils::perf::Perf;
use quakeworld::vfs::VfsFlattenedListEntry;
use quakeworld::vfs::{
    path::VfsPath, Vfs, VfsEntryDirectory, VfsEntryFile, VfsInternalNode, VfsList, VfsMetaData,
    VfsNode, VfsQueryDirectory, VfsQueryFile,
};

macro_rules! check_file {
    ($entry: expr, $file_name: expr, $file_size: expr) => {
        match $entry {
            quakeworld::vfs::VfsEntry::File(vfs_entry_file) => {
                assert!(vfs_entry_file.path.equals_string($file_name));
                assert_eq!(vfs_entry_file.meta.size, $file_size);
            }
            quakeworld::vfs::VfsEntry::Directory(vfs_entry_directory) => {
                assert!(false, "type should be file");
            }
        };
    };
}

macro_rules! check_directory {
    ($entry: expr, $directory_name: expr) => {
        match $entry {
            quakeworld::vfs::VfsEntry::File(vfs_entry_file) => {
                assert!(false, "type should be directory");
            }
            quakeworld::vfs::VfsEntry::Directory(vfs_entry_directory) => {
                assert!(vfs_entry_directory.path.equals_string($directory_name));
            }
        };
    };
}

macro_rules! create_pak {
    ($(($name: expr, $data: expr)), *) => {{
        let mut pak = quakeworld::pak::PakWriter::new();
        $(
            pak.file_add($name.to_string().into(), &$data[..]);
        )*
        match pak.write_data() {
            Ok(data) => std::io::Cursor::new(data),
            Err(e) => panic!("{}", e),
        }
    }};
}

macro_rules! create_pak_node {
    ($name: expr, $($rest:tt)*) => {
        {
            let pak_data = create_pak!($($rest)*);
            let pak = Pak::load($name, pak_data,
                    #[cfg(feature="trace")]
                    None
                ).unwrap();
            let pak_node = VfsInternalNode::new_from_pak(
            pak,
            VfsMetaData { ..Default::default() },);
            pak_node
        }
    };
}

#[ignore]
#[test]
pub fn directory_integration() -> Result<(), quakeworld::vfs::Error> {
    let mut vfs = Vfs::default();

    // mount the "tests"" directory under "some/path""
    let directory_path = std::path::Path::new("tests/");
    let vfs_mount_path = "some/path";

    let directory_node = VfsInternalNode::new_from_directory(directory_path.to_path_buf());
    let directory_node_hash = directory_node.hash().clone();
    vfs.insert_node(directory_node, vfs_mount_path);

    // check directory querying
    let directory_entry = VfsQueryDirectory::new("some", None);
    let de_path = directory_entry.clone();
    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.entries.len(), 1);
    assert_eq!(list.node_hash, *directory_node_hash);
    let entry = &list.entries[0];

    check_directory!(entry, "path");

    // check file reading from directory
    let data = vfs.read("some/path/data/file1.data", None)?;
    assert_eq!(data, b"deadbeef");

    let directory_entry = VfsQueryDirectory::new("", None);
    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.entries.len(), 1);
    let entry = &list.entries[0];
    check_directory!(entry, "some");
    Ok(())
}

#[ignore]
#[test]
pub fn file_integration() -> Result<(), quakeworld::vfs::Error> {
    let mut vfs = Vfs::default();

    // mount the "tests/data/file1.data" under "/"
    let file_path = std::path::Path::new("tests/data/file1.data");
    let file_node = VfsInternalNode::new_from_file(file_path.to_path_buf());
    let file_node_hash = file_node.hash().clone();
    vfs.insert_node(file_node, "");

    let directory_entry = VfsQueryDirectory::new("", None);

    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.node_hash, *file_node_hash);
    let entry = &list.entries[0];
    check_file!(entry, "file1.data", 8);

    // check file reading
    let file_path = VfsQueryFile::new("file1.data".try_into()?);
    let data = vfs.read(file_path, None)?;
    assert_eq!(data, b"deadbeef");

    let mut vfs = Vfs::default();

    // mount the "tests/data/file1.data" under "/some/path/"
    let file_path = std::path::Path::new("tests/data/file1.data");
    let file_node = VfsInternalNode::new_from_file(file_path.to_path_buf());
    let file_node_hash = file_node.hash().clone();
    vfs.insert_node(file_node, "some/path");

    let directory_entry = VfsQueryDirectory::new("", None);

    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.node_hash, *file_node_hash);
    let entry = &list.entries[0];
    check_directory!(entry, "some");

    let directory_entry = VfsQueryDirectory::new("some/path", None);

    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.node_hash, *file_node_hash);
    let entry = &list.entries[0];
    check_file!(entry, "file1.data", 8);

    // check file reading
    let data = vfs.read("some/path/file1.data", None)?;
    assert_eq!(data, b"deadbeef");
    Ok(())
}

#[test]
pub fn pak_integration() -> Result<(), quakeworld::vfs::Error> {
    let mut vfs = Vfs::default();
    let pak_node = create_pak_node!("test0.pak", ("testfile.dat", b"deadbeef"));
    let pak_node_hash = pak_node.hash().clone();
    vfs.insert_node(pak_node, "test");

    let directory_entry = VfsQueryDirectory::new("", None);

    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.node_hash, *pak_node_hash);
    let entry = &list.entries[0];
    check_directory!(entry, "test");

    let directory_entry = VfsQueryDirectory::new("test", None);

    let lists = vfs.list(directory_entry)?;
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(list.node_hash, *pak_node_hash);
    let entry = &list.entries[0];
    check_file!(entry, "testfile.dat", 8);
    let file_entry: VfsQueryFile = match &list.entries[0] {
        quakeworld::vfs::VfsEntry::File(vfs_entry_file) => vfs_entry_file.into(),
        quakeworld::vfs::VfsEntry::Directory(vfs_entry_directory) => {
            assert!(false, "type should have been directory");
            return Ok(());
        }
    };

    let data = vfs.read("test/testfile.dat", None)?;
    assert_eq!(data.len(), 8);
    assert_eq!(data, b"deadbeef");

    Ok(())
}

#[ignore]
#[test]
pub fn hash_integration() -> Result<(), quakeworld::vfs::Error> {
    let mut vfs = Vfs::default();

    let pak_node = create_pak_node!("test0.pak", ("testfile.dat", b"deadbeef"));
    let pak_0_node_hash = pak_node.hash().clone();
    vfs.insert_node(pak_node, "");

    let pak_node = create_pak_node!("test0_reverse.pak", ("testfile.dat", b"beefdead"));
    let pak_0_reverse_node_hash = pak_node.hash().clone();
    vfs.insert_node(pak_node, "");

    // mount the "tests/data/file1.data" under "/"
    let file_path = std::path::Path::new("tests/data/testfile.dat");
    let file_node = VfsInternalNode::new_from_file(file_path.to_path_buf());
    let file_node_hash = file_node.hash().clone();
    vfs.insert_node(file_node, "");

    let list = match vfs.list("") {
        Ok(l) => l,
        Err(e) => panic!("{}", e),
    };

    // check that flattening works
    let flattened_list = VfsFlattenedListEntry::flatten(list.clone());
    assert_eq!(flattened_list.len(), 1);
    for (i, hash) in vec![
        pak_0_node_hash.clone(),
        pak_0_reverse_node_hash.clone(),
        file_node_hash.clone(),
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(**hash, flattened_list[0].nodes[i]);
    }

    // check that getting data with a hash returns the right data
    for (i, (hash, binary)) in vec![
        (pak_0_node_hash, b"deadbeef"),
        (pak_0_reverse_node_hash, b"beefdead"),
        (file_node_hash, b"beerdead"),
    ]
    .iter()
    .enumerate()
    {
        let entry: &VfsList = &list[i];
        assert_eq!(entry.node_hash, *hash);
        let entry = &entry.entries[0];
        check_file!(entry, "testfile.dat", 8);
        let data = vfs.read("testfile.dat", Some(hash.clone()))?;
        assert_eq!(data, **binary);
    }
    Ok(())
}

#[test]
pub fn conversion_integration() -> Result<(), quakeworld::vfs::Error> {
    let path: VfsPath = "this/is/a/test".into();
    // conversion from path into String
    assert!(path.equals_string("this/is/a/test"));
    Ok(())
}

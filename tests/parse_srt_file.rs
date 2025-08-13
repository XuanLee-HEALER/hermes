use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
};

use blowup::sub::SrtFile;

fn main() {
    test_parse_srt();
}

#[test]
fn test_parse_srt() {
    let mut f = File::open("./test.srt").expect("failed to open the test srt file");
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .expect("failed to read the original file");
    f.seek(SeekFrom::Start(0))
        .expect("failed to seek the original file");
    let srt = SrtFile::read(f).unwrap();
    let mut nf = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("./test-copy.srt")
        .expect("failed to create the copy file");
    srt.write(&mut nf).expect("failed to write to copy file");
    nf.seek(SeekFrom::Start(0))
        .expect("failed to seek copy file");
    let mut o_buf = String::new();
    nf.read_to_string(&mut o_buf)
        .expect("failed to read the copy file");
    for diff in diff::lines(&buf, &o_buf) {
        match diff {
            diff::Result::Left(_) => panic!("Missing content in old file"),
            diff::Result::Both(_, _) => (),
            diff::Result::Right(_) => panic!("Added invalid content"),
        }
    }
}

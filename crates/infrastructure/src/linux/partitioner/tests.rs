use super::Partitioner;
use domain::PartitionScheme;
use std::io::Cursor;

#[test]
fn single_partition_aligns_to_1mib_and_fills() {
    let (start, len) = Partitioner::single_partition(64 * 1024 * 1024);
    assert_eq!(start, 1024 * 1024);
    assert_eq!(len % 512, 0);
    assert!(len >= 60 * 1024 * 1024);
}

#[test]
fn single_partition_zero_len_when_too_small() {
    let (_start, len) = Partitioner::single_partition(512 * 1024);
    assert_eq!(len, 0);
}

#[test]
fn gpt_writes_one_partition() {
    let total = 64 * 1024 * 1024u64;
    let mut dev = Cursor::new(vec![0u8; total as usize]);
    let (start, len) = Partitioner::single_partition(total);
    let (pstart, plen) = Partitioner::write_table(&mut dev, PartitionScheme::Gpt, start, len)
        .expect("escreve GPT");
    // O crate `gpt` posiciona na 1ª LBA usável; o offset real volta no retorno.
    assert!(pstart >= 512);
    assert!(plen > 0);
    // Releitura: deve haver exatamente 1 partição.
    let disk = gpt::GptConfig::new()
        .writable(false)
        .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
        .open_from_device(Box::new(&mut dev))
        .expect("reabre GPT");
    assert_eq!(disk.partitions().len(), 1);
}

#[test]
fn mbr_writes_one_fat32_partition() {
    let total = 64 * 1024 * 1024u64;
    let mut dev = Cursor::new(vec![0u8; total as usize]);
    let (start, len) = Partitioner::single_partition(total);
    let (pstart, plen) =
        Partitioner::write_table(&mut dev, PartitionScheme::Mbr, start, len).expect("escreve MBR");
    assert_eq!(pstart, start);
    assert_eq!(plen, len);
    let mbr = mbrman::MBR::read_from(&mut dev, 512).expect("reabre MBR");
    assert_eq!(mbr[1].sys, 0x0c);
    assert!(mbr[1].sectors > 0);
}

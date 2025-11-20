use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sample_guard::encryption::RFIDEncryption;

fn encryption_benchmark(c: &mut Criterion) {
    let encryption = RFIDEncryption::new(b"benchmark_key_32_bytes_long_for_aes!!");
    let data = vec![0u8; 128]; // Typical RFID payload size
    
    c.bench_function("encrypt_128_bytes", |b| {
        b.iter(|| {
            encryption.encrypt(black_box(&data)).unwrap()
        })
    });
    
    let ciphertext = encryption.encrypt(&data).unwrap();
    
    c.bench_function("decrypt_128_bytes", |b| {
        b.iter(|| {
            encryption.decrypt(black_box(&ciphertext)).unwrap()
        })
    });
}

fn hash_benchmark(c: &mut Criterion) {
    let encryption = RFIDEncryption::new(b"benchmark_key_32_bytes_long_for_aes!!");
    let data = vec![0u8; 128];
    
    c.bench_function("hash_128_bytes", |b| {
        b.iter(|| {
            encryption.hash(black_box(&data))
        })
    });
}

criterion_group!(benches, encryption_benchmark, hash_benchmark);
criterion_main!(benches);


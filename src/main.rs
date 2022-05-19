use std::env;
use std::process;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use std::str;

static r: [u64; 64] = 
    [7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,  7, 12, 17, 22,
    5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20,  5,  9, 14, 20, 
    4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23,  4, 11, 16, 23, 
    6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21,  6, 10, 15, 21];

static k: [u64; 64] = 
    [0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
    0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
    0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
    0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
    0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
    0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xea127fa, 0xd4ef3085, 0x04881d05,
    0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
    0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
    0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391];

/*
static mut k: [u64; 64] = [0; 64];

fn init_k() {
    let mut i: usize = 0;
    unsafe {
        while i < 64 {
            k[i] = (((i as f64) + 1f64).sin().abs().floor() * 2f64.powi(32)) as u64;
            i += 1;
        }
    }
}
*/

fn f_fun(x: u64, y: u64, z: u64) -> u64 {
    (x & y) | ( ((!x) & 0xffffffff) & z)
}

fn g_fun(x: u64, y: u64, z: u64) -> u64 {
    (x & z) | (y & ((!z) & 0xffffffff))
}

fn h_fun(x: u64, y: u64, z: u64) -> u64 {
    x ^ y ^ z
}

fn i_fun(x: u64, y: u64, z: u64) -> u64 {
    y ^ (x | ((!z) & 0xffffffff))
}

fn leftrotate (x: u64, c: u64) -> u64 {
    (x << c) | (x >> (32 - c))
}

fn cal_block(buf: &mut [u8], h0: &mut u64, h1: &mut u64, 
                h2: &mut u64, h3: &mut u64, len: usize, total_len: usize) {
    let mut a = *h0;
    let mut b = *h1;
    let mut c = *h2;
    let mut d = *h3;

    if len < 56 {
        buf[len] = 0x80;
        let mut idx = len + 1;
        while idx < 56 {
            buf[idx] = 0;
            idx += 1;
        }
        let len_bytes = total_len.to_le_bytes();
        while idx < 64 {
            buf[idx] = len_bytes[idx - 56];
            idx += 1;
        }
    }
    else if len < 64 {
        buf[len] = 0x80;
        let mut idx = len + 1;
        while idx < 64 {
            buf[idx] = 0;
            idx += 1;
        }
    }

    let mut w: [u64; 16] = [0; 16];
    for i in 0..16 {
        let temp: [u8; 8] = [0, 0, 0, 0, buf[i * 4], 
                            buf[i * 4 + 1], buf[i * 4 + 2],
                            buf[i * 4 + 3]];
        w[i] = u64::from_le_bytes(temp);
    }

    for i in 0..64 {
        let x: u64 = if i < 16 {
            leftrotate(a + f_fun(b, c, d) + k[i] + w[i], r[i])
        }
        else if i < 32 {
            leftrotate(a + g_fun(b, c, d) + k[i] + w[(5 * i + 1) % 16], r[i])
        }
        else if i < 48 {
            leftrotate(a + h_fun(b, c, d) + k[i] + w[(3 * i + 5) % 16], r[i])
        }
        else {
            leftrotate(a + i_fun(b, c, d) + k[i] + w[(7 * i) % 16], r[i])
        };

        let temp = d;
        d = c;
        c = b;
        b = (b + x) & 0xffffffff;
        a = temp;
    }

    *h0 = (*h0 + a) & 0xffffffff;
    *h1 = (*h1 + b) & 0xffffffff;
    *h2 = (*h2 + c) & 0xffffffff;
    *h3 = (*h3 + d) & 0xffffffff;

    if len == 56 {
        let mut buf2: [u8; 64] = [0; 64];
        let len_bytes = total_len.to_le_bytes();
        let mut idx = 56;
        while idx < 64 {
            buf[idx] = len_bytes[idx - 56];
            idx += 1;
        }
        cal_block(&mut buf2, h0, h1, h2, h3, 64, total_len);
    }
}

fn main() {
    // init_k();

    let args: Vec<String> = env::args().collect();

    let args_len = args.len();
    
    if args_len == 1 {
        eprintln!("TODO: not files by arguments");
        process::exit(0);
    }

    let mut file_num = 1;

    while file_num < args_len {
        let mut h0: u64 = 0x67452301;
        let mut h1: u64 = 0xefcdab89;
        let mut h2: u64 = 0x98badcfe;
        let mut h3: u64 = 0x10325476;

        let f = File::open(&args[file_num]);

        if f.is_err() {
            eprintln!("wrong file name");
            file_num += 1;
            continue;
        }

        let mut reader = BufReader::new(f.unwrap());
        let mut total_len = 0;
        loop {
            let mut buf: [u8; 64] = [0; 64];
            match reader.read(&mut buf) {
                Ok(len) => {
                    // println!("{}", str::from_utf8(&buf).unwrap());
                    total_len += len;

                    cal_block(&mut buf, &mut h0, &mut h1, 
                                &mut h2, &mut h3, len, total_len);
                    if len < 64 {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("file read: {}", e);
                    process::exit(-1);
                }
            }
        }

        let h0_bytes = h0.to_le_bytes();
        let h1_bytes = h1.to_le_bytes();
        let h2_bytes = h2.to_le_bytes();
        let h3_bytes = h3.to_le_bytes();

        println!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\
            {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}  {}",
                    h0_bytes[0], h0_bytes[1], h0_bytes[2], h0_bytes[3],
                    h1_bytes[0], h1_bytes[1], h1_bytes[2], h1_bytes[3],
                    h2_bytes[0], h2_bytes[1], h2_bytes[2], h2_bytes[3],
                    h3_bytes[0], h3_bytes[1], h3_bytes[2], h3_bytes[3],
                    args[file_num]);
        
        file_num += 1;
    }
}

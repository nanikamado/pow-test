#![allow(non_snake_case)]

pub struct Sha256Ctx {
    data: [u32; 64],
    datalen: u32,
    bitlen: u32,
    state: [u32; 8],
}

impl Default for Sha256Ctx {
    fn default() -> Self {
        Self {
            data: [0; 64],
            datalen: 0,
            bitlen: 0,
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
        }
    }
}

impl Sha256Ctx {
    pub fn to_array(&self) -> [u32; 74] {
        let mut r = [0; 74];
        r[..64].copy_from_slice(&self.data);
        r[64] = self.datalen;
        r[65] = self.bitlen;
        r[66..].copy_from_slice(&self.state);
        r
    }
}

fn ROTRIGHT(a: u32, b: u32) -> u32 {
    return ((a) >> (b)) | ((a) << (32 - (b)));
}
fn CH(x: u32, y: u32, z: u32) -> u32 {
    return ((x) & (y)) ^ (!(x) & (z));
}
fn MAJ(x: u32, y: u32, z: u32) -> u32 {
    return ((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z));
}
fn EP0(x: u32) -> u32 {
    return ROTRIGHT(x, 2) ^ ROTRIGHT(x, 13) ^ ROTRIGHT(x, 22);
}
fn EP1(x: u32) -> u32 {
    return ROTRIGHT(x, 6) ^ ROTRIGHT(x, 11) ^ ROTRIGHT(x, 25);
}
fn SIG0(x: u32) -> u32 {
    return ROTRIGHT(x, 7) ^ ROTRIGHT(x, 18) ^ ((x) >> 3);
}
fn SIG1(x: u32) -> u32 {
    return ROTRIGHT(x, 17) ^ ROTRIGHT(x, 19) ^ ((x) >> 10);
}

pub fn sha256_transform(ctx: &mut Sha256Ctx) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut d: u32;
    let mut e: u32;
    let mut f: u32;
    let mut g: u32;
    let mut h: u32;
    let mut i: usize = 0;
    let mut j = 0;
    let mut t1: u32;
    let mut t2: u32;
    let mut m: [u32; 64] = [0; 64];

    while i < 16 {
        m[i] = ((*ctx).data[j] << 24u32)
            | ((*ctx).data[j + 1] << 16)
            | ((*ctx).data[j + 2] << 8u32)
            | ((*ctx).data[j + 3]);
        i += 1;
        j += 4;
    }

    while i < 64 {
        m[i] = SIG1(m[i - 2])
            .wrapping_add(m[i - 7])
            .wrapping_add(SIG0(m[i - 15]))
            .wrapping_add(m[i - 16]);
        i += 1;
    }
    a = (*ctx).state[0];
    b = (*ctx).state[1];
    c = (*ctx).state[2];
    d = (*ctx).state[3];
    e = (*ctx).state[4];
    f = (*ctx).state[5];
    g = (*ctx).state[6];
    h = (*ctx).state[7];

    let k = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    for i in 0..64 {
        t1 = h
            .wrapping_add(EP1(e))
            .wrapping_add(CH(e, f, g))
            .wrapping_add(k[i])
            .wrapping_add(m[i]);
        t2 = EP0(a).wrapping_add(MAJ(a, b, c));
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    (*ctx).state[0] = (*ctx).state[0].wrapping_add(a);
    (*ctx).state[1] = (*ctx).state[1].wrapping_add(b);
    (*ctx).state[2] = (*ctx).state[2].wrapping_add(c);
    (*ctx).state[3] = (*ctx).state[3].wrapping_add(d);
    (*ctx).state[4] = (*ctx).state[4].wrapping_add(e);
    (*ctx).state[5] = (*ctx).state[5].wrapping_add(f);
    (*ctx).state[6] = (*ctx).state[6].wrapping_add(g);
    (*ctx).state[7] = (*ctx).state[7].wrapping_add(h);
}

pub fn sha256_push(ctx: &mut Sha256Ctx, n: u32) {
    (*ctx).data[(*ctx).datalen as usize] = n;
    (*ctx).datalen += 1;
    if (*ctx).datalen == 64 {
        sha256_transform(ctx);

        // if (*ctx).bitlen[0] > 0xffffffffu - (512u) {
        //     (*ctx).bitlen[1]++;
        // }
        (*ctx).bitlen += 512;

        (*ctx).datalen = 0;
    }
}

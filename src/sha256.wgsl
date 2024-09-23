struct SHA256_CTX {
    data: array<u32, 64>,
    datalen: u32,
    bitlen: array<u32, 2>,
    state: array<u32, 8>,
  };

@group(0) @binding(0) var<storage, read> input : array<u32>;
@group(0) @binding(1) var<storage, read> inputSize : array<u32>;
@group(0) @binding(2) var<storage, read_write> result : array<u32>;

const SHA256_BLOCK_SIZE = 32u;

fn log2_32(value: u32) -> u32 {
  var v = value;
    var tab32 = array<u32, 32> (
     0u,  9u,  1u, 10u, 13u, 21u,  2u, 29u,
    11u, 14u, 16u, 18u, 22u, 25u,  3u, 30u,
     8u, 12u, 20u, 28u, 15u, 17u, 24u,  7u,
    19u, 27u, 23u,  6u, 26u,  5u,  4u, 31u
    );
    v |= v >> 1u;
    v |= v >> 2u;
    v |= v >> 4u;
    v |= v >> 8u;
    v |= v >> 16u;
    return tab32[(v*0x07C4ACDDu) >> 27u];
}

fn ROTLEFT(a: u32, b: u32) -> u32 {return (((a) << (b)) | ((a) >> (32u - (b))));}
fn ROTRIGHT(a: u32, b: u32) -> u32 {return (((a) >> (b)) | ((a) << (32u - (b))));}

fn CH(x: u32, y: u32, z: u32) -> u32 {return (((x) & (y)) ^ (~(x) & (z)));}
fn MAJ(x: u32, y: u32, z: u32) -> u32 {return (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)));}
fn EP0(x: u32) -> u32 {return (ROTRIGHT(x, 2u) ^ ROTRIGHT(x, 13u) ^ ROTRIGHT(x, 22u));}
fn EP1(x: u32) -> u32 {return (ROTRIGHT(x, 6u) ^ ROTRIGHT(x, 11u) ^ ROTRIGHT(x, 25u));}
fn SIG0(x: u32) -> u32 {return (ROTRIGHT(x, 7u) ^ ROTRIGHT(x, 18u) ^ ((x) >> 3u));}
fn SIG1(x: u32) -> u32 {return (ROTRIGHT(x, 17u) ^ ROTRIGHT(x, 19u) ^ ((x) >> 10u));}

fn sha256_transform(ctx: ptr<function, SHA256_CTX>) {
    var a: u32;
    var b: u32;
    var c: u32;
    var d: u32;
    var e: u32;
    var f: u32;
    var g: u32;
    var h: u32;
    var i: i32 = 0;
    var j: u32 = 0u;
    var t1: u32;
    var t2: u32;
    var m: array<u32, 64> ;


    while i < 16 {
        m[i] = ((*ctx).data[j] << 24u) | ((*ctx).data[j + 1u] << 16u) | ((*ctx).data[j + 2u] << 8u) | ((*ctx).data[j + 3u]);
        i++;
        j += 4u;
    }

    while i < 64 {
        m[i] = SIG1(m[i - 2]) + m[i - 7] + SIG0(m[i - 15]) + m[i - 16];
        i++;
    }
    a = (*ctx).state[0];
    b = (*ctx).state[1];
    c = (*ctx).state[2];
    d = (*ctx).state[3];
    e = (*ctx).state[4];
    f = (*ctx).state[5];
    g = (*ctx).state[6];
    h = (*ctx).state[7];

    // for Deno bug
    var k = array<u32, 64>(
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u,
    );

    for (i = 0; i < 64; i++) {
        t1 = h + EP1(e) + CH(e, f, g) + k[i] + m[i];
        t2 = EP0(a) + MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }


    (*ctx).state[0] += a;
    (*ctx).state[1] += b;
    (*ctx).state[2] += c;
    (*ctx).state[3] += d;
    (*ctx).state[4] += e;
    (*ctx).state[5] += f;
    (*ctx).state[6] += g;
    (*ctx).state[7] += h;
}

fn sha256_push(ctx: ptr<function, SHA256_CTX>, n: u32) {
  (*ctx).data[(*ctx).datalen] = n;
        (*ctx).datalen++;
        if (*ctx).datalen == 64u {
            sha256_transform(ctx);

            if (*ctx).bitlen[0] > 0xffffffffu - (512u) {
                (*ctx).bitlen[1]++;
            }
            (*ctx).bitlen[0] += 512u;


            (*ctx).datalen = 0u;
        }
}

fn sha256_update(ctx: ptr<function, SHA256_CTX>, len: u32) {
    for (var i: u32 = 0u; i < len; i++) {
      sha256_push(ctx, input[i]);
    }
}

fn sha256_final(ctx: ptr<function, SHA256_CTX>, hash: ptr<function, array<u32, SHA256_BLOCK_SIZE>>) {
    var i: u32 = (*ctx).datalen;

    if (*ctx).datalen < 56u {
        (*ctx).data[i] = 0x80u;
        i++;
        while i < 56u {
            (*ctx).data[i] = 0x00u;
            i++;
        }
    } else {
        (*ctx).data[i] = 0x80u;
        i++;
        while i < 64u {
            (*ctx).data[i] = 0x00u;
            i++;
        }
        sha256_transform(ctx);
        for (var i = 0; i < 56 ; i++) {
            (*ctx).data[i] = 0u;
        }
    }


    if (*ctx).bitlen[0] > 0xffffffffu - (*ctx).datalen * 8u {
        (*ctx).bitlen[1]++;
    }
    (*ctx).bitlen[0] += (*ctx).datalen * 8u;


    (*ctx).data[63] = (*ctx).bitlen[0];
    (*ctx).data[62] = (*ctx).bitlen[0] >> 8u;
    (*ctx).data[61] = (*ctx).bitlen[0] >> 16u;
    (*ctx).data[60] = (*ctx).bitlen[0] >> 24u;
    (*ctx).data[59] = (*ctx).bitlen[1];
    (*ctx).data[58] = (*ctx).bitlen[1] >> 8u;
    (*ctx).data[57] = (*ctx).bitlen[1] >> 16u;
    (*ctx).data[56] = (*ctx).bitlen[1] >> 24u;
    sha256_transform(ctx);
}

fn push_string(ctx: ptr<function, SHA256_CTX>, n: u32) {
  var m = n;
  while m > 0u {
    sha256_push(ctx, m % 10u + 48u);
    m /= 10u;
  }
}

fn leading_zeros(ctx: ptr<function, SHA256_CTX>) -> u32 {
    var zeros = 0u;
    for (var i = 0u; i < 8u; i++) {
      let n = (*ctx).state[i];
      let z = 32u - log2_32(n);
      zeros += z;
      if z != 32 {
        return zeros;
      }
    }
    return zeros;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var ctx: SHA256_CTX;
    var buf: array<u32, SHA256_BLOCK_SIZE>;

    // CTX INIT
    ctx.datalen = 0u;
    ctx.bitlen[0] = 0u;
    ctx.bitlen[1] = 0u;
    ctx.state[0] = 0x6a09e667u;
    ctx.state[1] = 0xbb67ae85u;
    ctx.state[2] = 0x3c6ef372u;
    ctx.state[3] = 0xa54ff53au;
    ctx.state[4] = 0x510e527fu;
    ctx.state[5] = 0x9b05688cu;
    ctx.state[6] = 0x1f83d9abu;
    ctx.state[7] = 0x5be0cd19u;

    let i = global_id.x + global_id.y * 256 * 65535;

    sha256_update(&ctx, inputSize[0]);
    push_string(&ctx, i);
    sha256_final(&ctx, &buf);
    result[i / 4] |= leading_zeros(&ctx) << (i % 4) * 8;
}

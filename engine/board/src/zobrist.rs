pub struct ZobristKeys {
    pub pieces: [[[u64; 64]; 6]; 2],
    pub castling: [u64; 16],
    pub ep: [u64; 8],
    pub side: u64,
}

pub const KEYS: ZobristKeys = generate_keys();

const fn generate_keys() -> ZobristKeys {
    let mut state = 0x98f10715e2bf2f05u64;

    const fn rand(mut s: u64) -> (u64, u64) {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s, s)
    }

    let mut pieces = [[[0; 64]; 6]; 2];
    let mut color = 0;
    while color < 2 {
        let mut piece = 0;
        while piece < 6 {
            let mut sq = 0;
            while sq < 64 {
                let (val, new_state) = rand(state);
                state = new_state;
                pieces[color][piece][sq] = val;
                sq += 1;
            }
            piece += 1;
        }
        color += 1;
    }

    let mut castling = [0; 16];
    let mut i = 0;
    while i < 16 {
        let (val, new_state) = rand(state);
        state = new_state;
        castling[i] = val;
        i += 1;
    }

    let mut ep = [0; 8];
    let mut i = 0;
    while i < 8 {
        let (val, new_state) = rand(state);
        state = new_state;
        ep[i] = val;
        i += 1;
    }

    let (side, _) = rand(state);

    ZobristKeys { pieces, castling, ep, side }
}

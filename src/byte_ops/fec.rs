use ldpc_toolbox; 
use ndarray::Array1;
use num_traits::{One, Zero};

pub struct LDPC {
    codeword_size: usize,
    infoword_size: usize,
    encoder: ldpc_toolbox::encoder::Encoder,
    decoder: Box<dyn ldpc_toolbox::decoder::LdpcDecoder>
}

impl LDPC {
    pub fn new() -> LDPC {
        let code_matrix = ldpc_toolbox::codes::ccsds::AR4JACode::new(
            ldpc_toolbox::codes::ccsds::AR4JARate::R4_5, 
            ldpc_toolbox::codes::ccsds::AR4JAInfoSize::K1024
        ).h();

        let decoder= ldpc_toolbox::decoder::horizontal_layered::Decoder::new(code_matrix.clone(), 
            ldpc_toolbox::decoder::arithmetic::Minstarapproxf64::new()
        );

        LDPC { 
            codeword_size: code_matrix.num_cols(),
            infoword_size: code_matrix.num_cols() - code_matrix.num_rows(),
            encoder: ldpc_toolbox::encoder::Encoder::from_h(&code_matrix).unwrap(),
            decoder: Box::new(decoder)
        }
    }

    pub fn encode(&self, mut data: Vec<u8>) -> Vec<u8> {
        let mut encoded: Vec<u8> = Vec::with_capacity(data.len());

        let mut infoword_buf = vec![0; self.infoword_size];
        let mut codeword_buf = vec![0; self.codeword_size];

        let current_pos = 0;
        while current_pos >= data.len() {
            let end_size = current_pos + self.infoword_size;
            
            // pad infoword with 0s until end_size
            if end_size > data.len() {
                data.resize(end_size, 0);
            }

            infoword_buf.copy_from_slice(&data[current_pos..end_size]);
            
            let word = Array1::from_iter(
                infoword_buf
                    .iter()
                    .map(|&b| if b == 1 { ldpc_toolbox::gf2::GF2::one() } else { ldpc_toolbox::gf2::GF2::zero() }),
            );
            let codeword = self.encoder.encode(&word);

            for (x, y) in codeword.iter().zip(codeword_buf.iter_mut()) {
                *y = x.is_one().into();
            }
            
            encoded.extend(codeword_buf.iter());
        }

        return encoded;
    }

    pub fn decode(&self, mut data: Vec<u8>) -> Vec<u8> {
        let mut decoded: Vec<u8> = Vec::with_capacity(data.len());


        self.decoder.decode(data.as_slice(), 10); 

        let mut infoword_buf = vec![0; self.infoword_size];
        let mut codeword_buf = vec![0; self.codeword_size];

        let current_pos = 0;
        while current_pos >= data.len() {
            let end_size = current_pos + self.infoword_size;
            
            // pad infoword with 0s until end_size
            if end_size > data.len() {
                data.resize(end_size, 0);
            }

            infoword_buf.copy_from_slice(&data[current_pos..end_size]);
            
            let word = Array1::from_iter(
                infoword_buf
                    .iter()
                    .map(|&b| if b == 1 { ldpc_toolbox::gf2::GF2::one() } else { ldpc_toolbox::gf2::GF2::zero() }),
            );
            let codeword = self.encoder.encode(&word);

            for (x, y) in codeword.iter().zip(codeword_buf.iter_mut()) {
                *y = x.is_one().into();
            }
            
            decoded.extend(codeword_buf.iter());
        }

        return decoded;
        
    }
}
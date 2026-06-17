use nnnoiseless::{DenoiseState, FRAME_SIZE};

pub struct RnnoiseEffect {
    pub enabled: bool,
    state: Option<Box<DenoiseState<'static>>>,
    in_buf: Vec<f32>,
    out_buf: Vec<f32>,
}

impl Default for RnnoiseEffect {
    fn default() -> Self {
        Self {
            enabled: false,
            state: None,
            in_buf: Vec::with_capacity(FRAME_SIZE * 2),
            out_buf: Vec::with_capacity(FRAME_SIZE * 2),
        }
    }
}

impl RnnoiseEffect {
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        let state = self.state.get_or_insert_with(DenoiseState::new);

        // nnnoiseless ожидает значения в диапазоне i16 (-32768..32767)
        self.in_buf.extend(samples.iter().map(|&s| s * 32768.0));

        let mut frame_out = vec![0f32; FRAME_SIZE];

        while self.in_buf.len() >= FRAME_SIZE {
            let frame_in: Vec<f32> = self.in_buf.drain(..FRAME_SIZE).collect();
            state.process_frame(&mut frame_out, &frame_in);
            self.out_buf.extend(frame_out.iter().map(|&s| s / 32768.0));
        }

        let n = samples.len().min(self.out_buf.len());
        let drained: Vec<f32> = self.out_buf.drain(..n).collect();
        samples[..n].copy_from_slice(&drained);
        // Редкий случай: неполный фрейм — тишина (~10мс)
        for s in &mut samples[n..] {
            *s = 0.0;
        }
    }
}

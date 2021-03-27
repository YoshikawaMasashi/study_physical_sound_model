use super::filter::Filter;
use super::ring_buffer::RingBuffer;

struct DelayLine {
    history_buffer: RingBuffer<f32>,
    filters: Vec<Filter<f32>>,
}

impl DelayLine {
    fn new(size: usize, filters: Vec<Filter<f32>>) -> Self {
        DelayLine {
            history_buffer: RingBuffer::<f32>::new(size, 0.0),
            filters,
        }
    }

    fn do_delay(&mut self, input: f32) -> f32 {
        let mut x: f32 = *self.history_buffer.last();

        let filter_num = self.filters.len();
        for i in 0..filter_num {
            x = self.filters[i].filter(x);
        }

        self.history_buffer.push(input);

        x
    }
}

pub struct String {
    pub v_at_left_to_left: f32,
    pub v_at_right_to_left: f32,
    pub v_at_left_to_right: f32,
    pub v_at_right_to_right: f32,
    pub loadl: f32,
    pub loadr: f32,
    pub impedance: f32,
    to_left_delay_line: DelayLine,
    to_right_delay_line: DelayLine,
}

impl String {
    pub fn new(
        impedance: f32,
        del1: usize,
        del2: usize,
        left_filters: Vec<Filter<f32>>,
        right_filters: Vec<Filter<f32>>,
    ) -> String {
        String {
            v_at_left_to_left: 0.0,
            v_at_right_to_left: 0.0,
            v_at_left_to_right: 0.0,
            v_at_right_to_right: 0.0,
            loadl: 0.0,
            loadr: 0.0,
            impedance,
            to_left_delay_line: DelayLine::new(del1, left_filters),
            to_right_delay_line: DelayLine::new(del2, right_filters),
        }
    }

    pub fn do_delay(&mut self) {
        self.v_at_left_to_left = self.to_left_delay_line.do_delay(self.v_at_right_to_left);
        self.v_at_right_to_right = self.to_right_delay_line.do_delay(self.v_at_left_to_right);
    }
}

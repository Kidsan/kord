//! Model definition for the Kord model.

use core::f32;

use burn::{
    module::{Module, Param},
    nn::{self},
    tensor::{backend::Backend, Tensor},
};

use super::{helpers::Sigmoid, mlp::Mlp, INPUT_SPACE_SIZE, NUM_CLASSES};

#[cfg(feature = "ml_train")]
use crate::ml::train::{
    data::KordBatch,
    helpers::{KordClassificationOutput, MeanSquareLoss},
};

#[derive(Module, Debug)]
pub struct KordModel<B: Backend> {
    input: Param<nn::Linear<B>>,
    mlp: Param<Mlp<B>>,
    output: Param<nn::Linear<B>>,
    sigmoid: Sigmoid,
}

impl<B: Backend> KordModel<B> {
    pub fn new(mlp_layers: usize, mlp_size: usize, mlp_dropout: f64, sigmoid_strength: f32) -> Self {
        let input = nn::Linear::new(&nn::LinearConfig::new(INPUT_SPACE_SIZE, mlp_size));
        let mlp = Mlp::new(mlp_layers, mlp_size, mlp_dropout);
        let output = nn::Linear::new(&nn::LinearConfig::new(mlp_size, NUM_CLASSES));
        let sigmoid = Sigmoid::new(sigmoid_strength);

        Self {
            input: Param::from(input),
            mlp: Param::from(mlp),
            output: Param::from(output),
            sigmoid,
        }
    }

    pub fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        let mut x = input;

        x = self.input.forward(x);
        x = self.mlp.forward(x);
        x = self.output.forward(x);
        x = self.sigmoid.forward(x);

        x
    }

    #[cfg(feature = "ml_train")]
    pub fn forward_classification(&self, item: KordBatch<B>) -> KordClassificationOutput<B> {

        let targets = item.targets;
        let output = self.forward(item.samples);

        let loss = MeanSquareLoss::default();
        let loss = loss.forward(output.clone(), targets.clone());

        // let loss = BinaryCrossEntropyLoss::default();
        // let loss = loss.forward(output.clone(), targets.clone());

        // let mut loss = FocalLoss::default();
        // loss.gamma = 2.0;
        // let loss = loss.forward(output.clone(), targets.clone());

        //let loss = loss + l1_regularization(self, 1e-4);

        // let harmonic_penalty_tensor = get_harmonic_penalty_tensor().to_device(&output.device());
        // let harmonic_loss = output.clone().matmul(harmonic_penalty_tensor).sum_dim(0).mean().mul_scalar(0.0001);

        // let loss = loss + harmonic_loss;

        KordClassificationOutput { loss, output, targets }
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: Param<nn::conv::Conv1d<B>>,
    activation: nn::ReLU,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(in_channels: usize, out_channels: usize, kernel_size: usize) -> Self {
        let conv = nn::conv::Conv1d::new(
            &nn::conv::Conv1dConfig::new(in_channels, out_channels, kernel_size).with_bias(false),
        );

        Self {
            conv: Param::from(conv),
            activation: nn::ReLU::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 3>) -> Tensor<B, 3> {
        let x = self.conv.forward(input);
        self.activation.forward(x)
    }
}
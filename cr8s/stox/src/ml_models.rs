// ml_models.rs - Machine Learning extensions for your stock analysis system

use std::collections::HashMap;
use std::f64::consts::PI;
use crate::types::Candle;

// ============================================================================
// FEATURE ENGINEERING
// ============================================================================

#[derive(Debug, Clone)]
pub struct FeatureSet {
    pub features: Vec<f64>,
    pub labels: Vec<String>,
    pub target: Option<f64>, // For supervised learning
}

pub struct FeatureEngineer;

impl FeatureEngineer {
    /// Extract comprehensive features from candles and indicators
    pub fn extract_features(
        candles: &[Candle],
        indicator_map: &HashMap<String, Vec<Option<f64>>>,
        lookback_window: usize,
    ) -> Vec<FeatureSet> {
        let mut feature_sets = Vec::new();
        
        for i in lookback_window..candles.len() {
            let mut features = Vec::new();
            let mut labels = Vec::new();
            
            // Price-based features
            let current = &candles[i];
            let prev = &candles[i-1];
            
            // Returns and volatility
            let return_1d = (current.close - prev.close) / prev.close;
            let return_5d = if i >= 5 {
                (current.close - candles[i-5].close) / candles[i-5].close
            } else { 0.0 };
            
            // Volatility (rolling std of returns)
            let volatility = Self::calculate_volatility(&candles[i-lookback_window..=i]);
            
            // Volume features
            let volume_ratio = current.volume.unwrap_or(0.0) / 
                Self::average_volume(&candles[i-lookback_window..=i]);
            
            // Price position features
            let high_low_ratio = (current.close - current.low) / (current.high - current.low);
            let close_position = (current.close - current.open) / current.open;
            
            features.extend_from_slice(&[
                return_1d, return_5d, volatility, volume_ratio, 
                high_low_ratio, close_position
            ]);
            labels.extend_from_slice(&[
                "return_1d", "return_5d", "volatility", "volume_ratio",
                "high_low_ratio", "close_position"
            ].map(|s| s.to_string()));
            
            // Technical indicator features
            for (name, values) in indicator_map {
                if let Some(Some(value)) = values.get(i) {
                    features.push(*value);
                    labels.push(name.clone());
                }
            }
            
            // Pattern features
            let pattern_features = Self::extract_pattern_features(&candles[i-lookback_window..=i]);
            features.extend(pattern_features.0);
            labels.extend(pattern_features.1);
            
            // Target: next day return for prediction
            let target = if i + 1 < candles.len() {
                Some((candles[i+1].close - current.close) / current.close)
            } else {
                None
            };
            
            feature_sets.push(FeatureSet { features, labels, target });
        }
        
        feature_sets
    }
    
    fn calculate_volatility(candles: &[Candle]) -> f64 {
        if candles.len() < 2 { return 0.0; }
        
        let returns: Vec<f64> = candles.windows(2)
            .map(|w| (w[1].close - w[0].close) / w[0].close)
            .collect();
        
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        variance.sqrt()
    }
    
    fn average_volume(candles: &[Candle]) -> f64 {
        let sum: f64 = candles.iter()
            .map(|c| c.volume.unwrap_or(0.0))
            .sum();
        sum / candles.len() as f64
    }
    
    fn extract_pattern_features(candles: &[Candle]) -> (Vec<f64>, Vec<String>) {
        let mut features = Vec::new();
        let mut labels = Vec::new();
        
        if candles.len() >= 3 {
            // Trend strength
            let trend = Self::calculate_trend_strength(candles);
            features.push(trend);
            labels.push("trend_strength".to_string());
            
            // Support/Resistance levels
            let (support, resistance) = Self::find_support_resistance(candles);
            let support_distance = (candles.last().unwrap().close - support) / support;
            let resistance_distance = (resistance - candles.last().unwrap().close) / resistance;
            
            features.extend_from_slice(&[support_distance, resistance_distance]);
            labels.extend_from_slice(&["support_distance", "resistance_distance"]
                .map(|s| s.to_string()));
        }
        
        (features, labels)
    }
    
    fn calculate_trend_strength(candles: &[Candle]) -> f64 {
        if candles.len() < 3 { return 0.0; }
        
        let prices: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let n = prices.len() as f64;
        
        // Linear regression slope
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = prices.iter().sum::<f64>() / n;
        
        let numerator: f64 = (0..prices.len())
            .map(|i| (i as f64 - x_mean) * (prices[i] - y_mean))
            .sum();
        
        let denominator: f64 = (0..prices.len())
            .map(|i| (i as f64 - x_mean).powi(2))
            .sum();
        
        if denominator != 0.0 { numerator / denominator } else { 0.0 }
    }
    
    fn find_support_resistance(candles: &[Candle]) -> (f64, f64) {
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        
        let support = lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let resistance = highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        (support, resistance)
    }
}

// ============================================================================
// PREDICTION MODELS
// ============================================================================

pub trait MLModel {
    fn train(&mut self, features: &[FeatureSet]) -> Result<(), String>;
    fn predict(&self, features: &[f64]) -> Result<f64, String>;
    fn get_feature_importance(&self) -> Vec<(String, f64)>;
}

// Simple Linear Regression
pub struct LinearRegression {
    weights: Vec<f64>,
    bias: f64,
    feature_names: Vec<String>,
    learning_rate: f64,
    epochs: usize,
}

impl LinearRegression {
    pub fn new(learning_rate: f64, epochs: usize) -> Self {
        Self {
            weights: Vec::new(),
            bias: 0.0,
            feature_names: Vec::new(),
            learning_rate,
            epochs,
        }
    }
}

impl MLModel for LinearRegression {
    fn train(&mut self, feature_sets: &[FeatureSet]) -> Result<(), String> {
        if feature_sets.is_empty() {
            return Err("No training data provided".to_string());
        }
        
        let n_features = feature_sets[0].features.len();
        self.weights = vec![0.0; n_features];
        self.feature_names = feature_sets[0].labels.clone();
        
        // Gradient descent
        for _ in 0..self.epochs {
            let mut weight_gradients = vec![0.0; n_features];
            let mut bias_gradient = 0.0;
            
            for fs in feature_sets {
                if let Some(target) = fs.target {
                    let prediction = self.predict(&fs.features).unwrap_or(0.0);
                    let error = prediction - target;
                    
                    // Update gradients
                    for (i, &feature) in fs.features.iter().enumerate() {
                        weight_gradients[i] += error * feature;
                    }
                    bias_gradient += error;
                }
            }
            
            // Update weights
            let n_samples = feature_sets.len() as f64;
            for i in 0..n_features {
                self.weights[i] -= self.learning_rate * weight_gradients[i] / n_samples;
            }
            self.bias -= self.learning_rate * bias_gradient / n_samples;
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &[f64]) -> Result<f64, String> {
        if features.len() != self.weights.len() {
            return Err("Feature dimension mismatch".to_string());
        }
        
        let prediction = features.iter()
            .zip(&self.weights)
            .map(|(f, w)| f * w)
            .sum::<f64>() + self.bias;
        
        Ok(prediction)
    }
    
    fn get_feature_importance(&self) -> Vec<(String, f64)> {
        self.feature_names.iter()
            .zip(&self.weights)
            .map(|(name, &weight)| (name.clone(), weight.abs()))
            .collect()
    }
}

// Random Forest (simplified implementation)
pub struct RandomForest {
    trees: Vec<DecisionTree>,
    n_trees: usize,
    max_depth: usize,
    feature_names: Vec<String>,
}

impl RandomForest {
    pub fn new(n_trees: usize, max_depth: usize) -> Self {
        Self {
            trees: Vec::new(),
            n_trees,
            max_depth,
            feature_names: Vec::new(),
        }
    }
}

impl MLModel for RandomForest {
    fn train(&mut self, feature_sets: &[FeatureSet]) -> Result<(), String> {
        if feature_sets.is_empty() {
            return Err("No training data provided".to_string());
        }
        
        self.feature_names = feature_sets[0].labels.clone();
        self.trees.clear();
        
        for _ in 0..self.n_trees {
            let mut tree = DecisionTree::new(self.max_depth);
            
            // Bootstrap sampling
            let bootstrap_sample = Self::bootstrap_sample(feature_sets);
            tree.train(&bootstrap_sample)?;
            
            self.trees.push(tree);
        }
        
        Ok(())
    }
    
    fn predict(&self, features: &[f64]) -> Result<f64, String> {
        if self.trees.is_empty() {
            return Err("Model not trained".to_string());
        }
        
        let predictions: Result<Vec<f64>, String> = self.trees.iter()
            .map(|tree| tree.predict(features))
            .collect();
        
        let predictions = predictions?;
        let average = predictions.iter().sum::<f64>() / predictions.len() as f64;
        
        Ok(average)
    }
    
    fn get_feature_importance(&self) -> Vec<(String, f64)> {
        let mut importance_sum = vec![0.0; self.feature_names.len()];
        
        for tree in &self.trees {
            let tree_importance = tree.get_feature_importance();
            for (name, importance) in tree_importance {
                if let Some(index) = self.feature_names.iter().position(|n| n == &name) {
                    importance_sum[index] += importance;
                }
            }
        }
        
        self.feature_names.iter()
            .zip(importance_sum)
            .map(|(name, importance)| (name.clone(), importance / self.n_trees as f64))
            .collect()
    }
}

impl RandomForest {
    fn bootstrap_sample(feature_sets: &[FeatureSet]) -> Vec<FeatureSet> {
        let mut rng = SimpleRng::new(42); // Simple PRNG
        let mut sample = Vec::new();
        
        for _ in 0..feature_sets.len() {
            let index = rng.next_usize() % feature_sets.len();
            sample.push(feature_sets[index].clone());
        }
        
        sample
    }
}

// Simple Decision Tree
pub struct DecisionTree {
    root: Option<TreeNode>,
    max_depth: usize,
    feature_names: Vec<String>,
}

#[derive(Clone)]
struct TreeNode {
    feature_index: Option<usize>,
    threshold: Option<f64>,
    value: Option<f64>,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl DecisionTree {
    pub fn new(max_depth: usize) -> Self {
        Self {
            root: None,
            max_depth,
            feature_names: Vec::new(),
        }
    }
    
    fn build_tree(&self, data: &[FeatureSet], depth: usize) -> Option<TreeNode> {
        if data.is_empty() || depth >= self.max_depth {
            return None;
        }
        
        // Calculate mean target value
        let targets: Vec<f64> = data.iter()
            .filter_map(|fs| fs.target)
            .collect();
        
        if targets.is_empty() {
            return None;
        }
        
        let mean_target = targets.iter().sum::<f64>() / targets.len() as f64;
        
        // If all targets are the same or max depth reached, create leaf
        if targets.iter().all(|&t| (t - mean_target).abs() < 1e-6) || depth >= self.max_depth {
            return Some(TreeNode {
                feature_index: None,
                threshold: None,
                value: Some(mean_target),
                left: None,
                right: None,
            });
        }
        
        // Find best split
        let (best_feature, best_threshold) = self.find_best_split(data)?;
        
        // Split data
        let (left_data_refs, right_data_refs): (Vec<&FeatureSet>, Vec<&FeatureSet>) = data.iter()
            .partition(|fs| fs.features.get(best_feature).unwrap_or(&0.0) <= &best_threshold);

        // Convert from Vec<&FeatureSet> to Vec<FeatureSet> by cloning
        let left_data: Vec<FeatureSet> = left_data_refs.into_iter().cloned().collect();
        let right_data: Vec<FeatureSet> = right_data_refs.into_iter().cloned().collect();
 
        Some(TreeNode {
            feature_index: Some(best_feature),
            threshold: Some(best_threshold),
            value: None,
            left: self.build_tree(&left_data, depth + 1).map(Box::new),
            right: self.build_tree(&right_data, depth + 1).map(Box::new),
        })
    }
    
    fn find_best_split(&self, data: &[FeatureSet]) -> Option<(usize, f64)> {
        let n_features = data[0].features.len();
        let mut best_score = f64::INFINITY;
        let mut best_split = None;
        
        for feature_idx in 0..n_features {
            let mut feature_values: Vec<f64> = data.iter()
                .map(|fs| fs.features.get(feature_idx).unwrap_or(&0.0))
                .cloned()
                .collect();
            
            feature_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            feature_values.dedup();
            
            for &threshold in &feature_values {
                let score = self.calculate_split_score(data, feature_idx, threshold);
                if score < best_score {
                    best_score = score;
                    best_split = Some((feature_idx, threshold));
                }
            }
        }
        
        best_split
    }
    
    fn calculate_split_score(&self, data: &[FeatureSet], feature_idx: usize, threshold: f64) -> f64 {
        let (left, right): (Vec<_>, Vec<_>) = data.iter()
            .partition(|fs| fs.features.get(feature_idx).unwrap_or(&0.0) <= &threshold);
        
        if left.is_empty() || right.is_empty() {
            return f64::INFINITY;
        }
        
        let left_variance = self.calculate_variance(&left);
        let right_variance = self.calculate_variance(&right);
        let left_weight = left.len() as f64 / data.len() as f64;
        let right_weight = right.len() as f64 / data.len() as f64;
        
        left_weight * left_variance + right_weight * right_variance
    }
    
    fn calculate_variance(&self, data: &[&FeatureSet]) -> f64 {
        let targets: Vec<f64> = data.iter()
            .filter_map(|fs| fs.target)
            .collect();
        
        if targets.len() <= 1 {
            return 0.0;
        }
        
        let mean = targets.iter().sum::<f64>() / targets.len() as f64;
        let variance = targets.iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f64>() / targets.len() as f64;
        
        variance
    }
}

impl MLModel for DecisionTree {
    fn train(&mut self, feature_sets: &[FeatureSet]) -> Result<(), String> {
        if feature_sets.is_empty() {
            return Err("No training data provided".to_string());
        }
        
        self.feature_names = feature_sets[0].labels.clone();
        self.root = self.build_tree(feature_sets, 0);
        
        Ok(())
    }
    
    fn predict(&self, features: &[f64]) -> Result<f64, String> {
        match &self.root {
            Some(node) => Ok(self.predict_node(node, features)),
            None => Err("Model not trained".to_string()),
        }
    }
    
    fn get_feature_importance(&self) -> Vec<(String, f64)> {
        // Simplified feature importance based on usage count
        let mut importance = vec![0.0; self.feature_names.len()];
        self.calculate_feature_importance(&self.root, &mut importance);
        
        self.feature_names.iter()
            .zip(importance)
            .map(|(name, imp)| (name.clone(), imp))
            .collect()
    }
}

impl DecisionTree {
    fn predict_node(&self, node: &TreeNode, features: &[f64]) -> f64 {
        match &node.value {
            Some(value) => *value,
            None => {
                let feature_value = features.get(node.feature_index.unwrap()).unwrap_or(&0.0);
                let threshold = node.threshold.unwrap();
                
                if *feature_value <= threshold {
                    match &node.left {
                        Some(left) => self.predict_node(left, features),
                        None => 0.0,
                    }
                } else {
                    match &node.right {
                        Some(right) => self.predict_node(right, features),
                        None => 0.0,
                    }
                }
            }
        }
    }
    
    fn calculate_feature_importance(&self, node: &Option<TreeNode>, importance: &mut [f64]) {
        if let Some(n) = node {
            if let Some(feature_idx) = n.feature_index {
                importance[feature_idx] += 1.0;
            }
            self.calculate_feature_importance(&n.left.as_deref().cloned(), importance);
            self.calculate_feature_importance(&n.right.as_deref().cloned(), importance);
        }
    }
}

// ============================================================================
// CLUSTERING
// ============================================================================

#[derive(Debug, Clone)]
pub struct Cluster {
    pub center: Vec<f64>,
    pub points: Vec<usize>,
}

pub struct KMeans {
    k: usize,
    max_iterations: usize,
    clusters: Vec<Cluster>,
}

impl KMeans {
    pub fn new(k: usize, max_iterations: usize) -> Self {
        Self {
            k,
            max_iterations,
            clusters: Vec::new(),
        }
    }
    
    pub fn fit(&mut self, data: &[Vec<f64>]) -> Result<(), String> {
        if data.is_empty() || data[0].is_empty() {
            return Err("Empty data provided".to_string());
        }
        
        let n_features = data[0].len();
        let mut rng = SimpleRng::new(42);
        
        // Initialize clusters randomly
        self.clusters = (0..self.k)
            .map(|_| {
                let center = (0..n_features)
                    .map(|_| rng.next_f64() * 10.0 - 5.0)
                    .collect();
                Cluster { center, points: Vec::new() }
            })
            .collect();
        
        for _ in 0..self.max_iterations {
            // Clear previous assignments
            for cluster in &mut self.clusters {
                cluster.points.clear();
            }
            
            // Assign points to clusters
            for (point_idx, point) in data.iter().enumerate() {
                let closest_cluster = self.find_closest_cluster(point);
                self.clusters[closest_cluster].points.push(point_idx);
            }
            
            // Update cluster centers
            for cluster in &mut self.clusters {
                if !cluster.points.is_empty() {
                    for (dim, center_val) in cluster.center.iter_mut().enumerate() {
                        *center_val = cluster.points.iter()
                            .map(|&idx| data[idx][dim])
                            .sum::<f64>() / cluster.points.len() as f64;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn predict(&self, point: &[f64]) -> usize {
        self.find_closest_cluster(point)
    }
    
    pub fn get_clusters(&self) -> &[Cluster] {
        &self.clusters
    }
    
    fn find_closest_cluster(&self, point: &[f64]) -> usize {
        self.clusters.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = Self::euclidean_distance(point, &a.center);
                let dist_b = Self::euclidean_distance(point, &b.center);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

// Simple PRNG for reproducible results without external deps
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }
    
    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
    
    fn next_usize(&mut self) -> usize {
        self.next() as usize
    }
}

// Model evaluation utilities
pub struct ModelEvaluator;

impl ModelEvaluator {
    pub fn mean_squared_error(actual: &[f64], predicted: &[f64]) -> f64 {
        actual.iter()
            .zip(predicted)
            .map(|(a, p)| (a - p).powi(2))
            .sum::<f64>() / actual.len() as f64
    }
    
    pub fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> f64 {
        actual.iter()
            .zip(predicted)
            .map(|(a, p)| (a - p).abs())
            .sum::<f64>() / actual.len() as f64
    }
    
    pub fn r_squared(actual: &[f64], predicted: &[f64]) -> f64 {
        let mean_actual = actual.iter().sum::<f64>() / actual.len() as f64;
        
        let ss_res: f64 = actual.iter()
            .zip(predicted)
            .map(|(a, p)| (a - p).powi(2))
            .sum();
        
        let ss_tot: f64 = actual.iter()
            .map(|a| (a - mean_actual).powi(2))
            .sum();
        
        1.0 - (ss_res / ss_tot)
    }
    
    pub fn sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let excess_return = mean_return - risk_free_rate;
        
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        
        let std_dev = variance.sqrt();
        
        if std_dev != 0.0 {
            excess_return / std_dev
        } else {
            0.0
        }
    }
    
    pub fn maximum_drawdown(prices: &[f64]) -> f64 {
        let mut max_drawdown = 0.0;
        let mut peak = prices[0];
        
        for &price in prices.iter().skip(1) {
            if price > peak {
                peak = price;
            }
            let drawdown = (peak - price) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        
        max_drawdown
    }
}

// ============================================================================
// INTEGRATION WITH YOUR EXISTING SYSTEM
// ============================================================================

pub struct MLPipeline {
    pub feature_engineer: FeatureEngineer,
    pub model: Box<dyn MLModel>,
    pub kmeans: Option<KMeans>,
}

impl MLPipeline {
    pub fn new(model: Box<dyn MLModel>) -> Self {
        Self {
            feature_engineer: FeatureEngineer,
            model,
            kmeans: None,
        }
    }
    
    pub fn train_prediction_model(
        &mut self,
        candles: &[Candle],
        indicator_map: &HashMap<String, Vec<Option<f64>>>,
        lookback_window: usize,
    ) -> Result<(), String> {
        let features = FeatureEngineer::extract_features(candles, indicator_map, lookback_window);
        self.model.train(&features)
    }
    
    pub fn predict_next_return(
        &self,
        candles: &[Candle],
        indicator_map: &HashMap<String, Vec<Option<f64>>>,
    ) -> Result<f64, String> {
        if candles.is_empty() {
            return Err("No candles provided".to_string());
        }
        
        let features = FeatureEngineer::extract_features(candles, indicator_map, 10);
        if let Some(last_features) = features.last() {
            self.model.predict(&last_features.features)
        } else {
            Err("Could not extract features".to_string())
        }
    }
    
    pub fn cluster_stocks(
        &mut self,
        all_features: &[Vec<f64>],
        n_clusters: usize,
    ) -> Result<Vec<usize>, String> {
        let mut kmeans = KMeans::new(n_clusters, 100);
        kmeans.fit(all_features)?;
        
        let assignments = all_features.iter()
            .map(|features| kmeans.predict(features))
            .collect();
        
        self.kmeans = Some(kmeans);
        Ok(assignments)
    }
    
    pub fn get_feature_importance(&self) -> Vec<(String, f64)> {
        self.model.get_feature_importance()
    }
}

// Example usage functions to integrate with your main.rs
pub fn run_ml_analysis(
    candles: &[Candle],
    indicator_map: &HashMap<String, Vec<Option<f64>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create ML pipeline with Random Forest
    let mut pipeline = MLPipeline::new(Box::new(RandomForest::new(10, 5)));
    
    // 2. Train prediction model
    pipeline.train_prediction_model(candles, indicator_map, 20)?;
    
    // 3. Make prediction
    let predicted_return = pipeline.predict_next_return(candles, indicator_map)?;
    println!("Predicted next day return: {:.4}%", predicted_return * 100.0);
    
    // 4. Show feature importance
    let importance = pipeline.get_feature_importance();
    println!("Top 5 most important features:");
    for (name, score) in importance.iter().take(5) {
        println!("  {}: {:.4}", name, score);
    }
    
    Ok(())
}
# 异构计算

在 Valkyrie 中，异构计算通过统一的数组抽象实现跨设备的高性能计算，专注于传统机器学习算法的加速。

## 数组类型体系

- `Array` 是一段内存中的数据
- `Array<T, N>` 是 `[T; N]` 的语法糖，表示固定大小数组
- `ArrayND` 是异构计算、机器学习的基础多维数组类型
- `Array1D` 是 `ArrayND` 的 type alias，专门用于一维数组操作

ArrayND 可以选择不同的设备 (device) 和布局 (layout)，系统会自动进行优化。

## 基本用法

### 创建数组

```valkyrie
# 创建不同维度的数组
let vector = Array1D::zeros(1000)           # 一维数组
let matrix = ArrayND::zeros([1000, 1000])   # 二维数组
let tensor = ArrayND::zeros([32, 3, 224, 224])  # 四维张量

# 从数据创建
let data = [1.0, 2.0, 3.0, 4.0]
let arr = Array1D::from(data)
let reshaped = arr.reshape([2, 2])  # 重塑为2x2矩阵
```

### 设备选择

```valkyrie
# 在不同设备上创建数组
let cpu_array = ArrayND::zeros([1024, 1024]).on_device(Device::CPU)
let gpu_array = ArrayND::zeros([1024, 1024]).on_device(Device::GPU)

# 设备间转换
let gpu_result = cpu_array.to_device(Device::GPU)
let cpu_result = gpu_array.to_cpu()
```

## 布局优化

### 自动布局选择

```valkyrie
# 系统会根据操作自动选择最优布局
let a = ArrayND::random([1024, 1024])
let b = ArrayND::random([1024, 1024])

# 矩阵乘法会自动选择最优的内存布局
let result = a.matmul(&b)  # 自动优化

# 手动指定布局（通常不需要）
let row_major = a.with_layout(Layout::RowMajor)
let col_major = a.with_layout(Layout::ColMajor)
```

### 异步数据传输

```valkyrie
# 异步设备间传输
async micro process_large_data() {
    let data = ArrayND::load("dataset.bin")
    let gpu_data = data.to_device_async(Device::GPU).await
    
    # GPU计算
    let result = gpu_data.matmul(&gpu_data.transpose())
    
    # 传回CPU保存
    let cpu_result = result.to_cpu_async().await
    cpu_result.save("result.bin")
}
```

## 常用操作

### 数学运算

```valkyrie
# 基本运算（自动选择最优设备和布局）
let 𝐀 = ArrayND::random([1000, 1000])
let 𝐁 = ArrayND::random([1000, 1000])

# 矩阵运算
let sum = 𝐀 + 𝐁
let product = 𝐀 ⊙ 𝐁  # 逐元素乘法（Hadamard积）
let matmul = 𝐀 · 𝐁     # 矩阵乘法
let transpose = 𝐀ᵀ

# 统计运算
let μ = 𝐀.mean()      # 均值
let Σ = 𝐀.sum()       # 求和
let max_val = max(𝐀)  # 最大值
let σ = 𝐀.std()       # 标准差
```

## 机器学习应用

### 线性回归

```valkyrie
# 线性回归模型
class LinearRegression {
    weights: ArrayND,
    bias: f32,
}

impl LinearRegression {
    micro new(n_features: usize) -> Self {
        Self {
            weights: ArrayND::zeros([n_features]),
            bias: 0.0,
        }
    }
    
    micro fit(&mut self, 𝐗: &ArrayND, 𝐲: &ArrayND, α: f32, epochs: usize) {
        let n = 𝐗.shape()[0] as f32  # 样本数量
        
        for _ in 0..epochs {
            # 预测
            let ŷ = 𝐗 · &self.weights + self.bias
            
            # 计算梯度
            let ε = ŷ - 𝐲  # 误差
            let ∇w = 𝐗ᵀ · &ε / n  # 权重梯度
            let ∇b = ε.mean()      # 偏置梯度
            
            # 更新参数（梯度下降）
            self.weights = &self.weights - α * &∇w
            self.bias -= α * ∇b
        }
    }
    
    micro predict(&self, 𝐗: &ArrayND) -> ArrayND {
        𝐗 · &self.weights + self.bias
    }
}
```

### 支持向量机 (SVM)

```valkyrie
# SVM 分类器
class SVM {
    weights: ArrayND,
    bias: f32,
    C: f32,  # 正则化参数
}

impl SVM {
    micro new(n_features: usize, C: f32) -> Self {
        Self {
            weights: ArrayND::zeros([n_features]),
            bias: 0.0,
            C,
        }
    }
    
    micro fit(&mut self, 𝐗: &ArrayND, 𝐲: &ArrayND, α: f32, epochs: usize) {
        for _ in 0..epochs {
            for i in 0..𝐗.shape()[0] {
                let 𝐱ᵢ = 𝐗.row(i)
                let yᵢ = 𝐲[i]
                
                let decision = 𝐱ᵢ · &self.weights + self.bias
                
                if yᵢ * decision < 1.0 {
                    # 支持向量，更新参数
                    self.weights = &self.weights + α * (yᵢ * &𝐱ᵢ - 2.0 * self.C * &self.weights)
                    self.bias += α * yᵢ
                } else {
                    # 正确分类，只应用正则化
                    self.weights = &self.weights - α * 2.0 * self.C * &self.weights
                }
            }
        }
    }
    
    micro predict(&self, 𝐗: &ArrayND) -> ArrayND {
        sign(𝐗 · &self.weights + self.bias)
    }
}
```

## 多设备并行

### K-Means 聚类并行化

```valkyrie
# 并行 K-Means 聚类
class ParallelKMeans {
    centroids: ArrayND,
    k: usize,
    devices: Vector<Device>,
}

impl ParallelKMeans {
    micro fit(&mut self, data: &ArrayND, max_iters: usize) {
        let n_devices = self.devices.len()
        let chunk_size = data.shape()[0] / n_devices
        
        for _ in 0..max_iters {
            # 并行计算距离和分配
            let assignments: Vector<ArrayND> = data.chunks(chunk_size)
                .zip(&self.devices)
                .par_iter()
                .map(|(chunk, device)| {
                    let chunk_gpu = chunk.to_device(device)
                    let centroids_gpu = self.centroids.to_device(device)
                    
                    # 计算距离矩阵
                    let distances = chunk_gpu.cdist(&centroids_gpu)
                    distances.argmin(1)  # 最近质心索引
                })
                .collect()
            
            # 更新质心
            self.update_centroids(data, &assignments)
        }
    }
    
    micro update_centroids(&mut self, data: &ArrayND, assignments: &[ArrayND]) {
        for k in 0..self.k {
            let mask = assignments.iter()
                .map(|assign| assign.eq(k))
                .reduce(|acc, x| acc.concat(&x))
                .unwrap()
            
            let cluster_points = data.masked_select(&mask)
            if cluster_points.shape()[0] > 0 {
                self.centroids.row_mut(k).copy_from(&cluster_points.mean(0))
            }
        }
    }
}
```

### 随机森林并行化

```valkyrie
# 并行随机森林
class ParallelRandomForest {
    trees: Vector<DecisionTree>,
    n_trees: usize,
}

impl ParallelRandomForest {
    micro fit(&mut self, X: &ArrayND, y: &ArrayND) {
        # 并行训练决策树
        self.trees = (0..self.n_trees)
            .into_par_iter()
            .map(|_| {
                # 自助采样
                let (X_sample, y_sample) = bootstrap_sample(X, y)
                
                # 训练单棵树
                let mut tree = DecisionTree::new()
                tree.fit(&X_sample, &y_sample)
                tree
            })
            .collect()
    }
    
    micro predict(&self, X: &ArrayND) -> ArrayND {
        # 并行预测
        let predictions: Vector<ArrayND> = self.trees
            .par_iter()
            .map(|tree| tree.predict(X))
            .collect()
        
        # 投票聚合
        majority_vote(&predictions)
    }
}
```

## 内存优化策略

### 数据分块处理

```valkyrie
# 大数据集分块处理
class ChunkedProcessor {
    chunk_size: usize,
    memory_limit: usize,
}

impl ChunkedProcessor {
    micro process_large_dataset(&self, data: &ArrayND, algorithm: &dyn Algorithm) -> ArrayND {
        let total_size = data.shape()[0]
        let mut results = Vec::new()
        
        for start in (0..total_size).step_by(self.chunk_size) {
            let end = (start + self.chunk_size).min(total_size)
            let chunk = data.slice([start..end, ..])
            
            # 处理单个数据块
            let chunk_result = algorithm.process(&chunk)
            results.push(chunk_result)
            
            # 检查内存使用
            if self.get_memory_usage() > self.memory_limit {
                self.gc_collect()  # 强制垃圾回收
            }
        }
        
        # 合并结果
        ArrayND::concat(&results, 0)
    }
    
    micro get_memory_usage(&self) -> usize {
        # 获取当前内存使用量
        std::mem::size_of_val(self) + self.estimate_array_memory()
    }
}
```

### 内存使用检查

```valkyrie
# 简单的内存使用监控
micro check_memory_usage() {
    let cpu_arrays = ArrayND::get_cpu_memory_usage()
    let gpu_arrays = ArrayND::get_gpu_memory_usage()
    
    println!("CPU 内存使用: {:.2} MB", cpu_arrays as f64 / 1024.0 / 1024.0)
    println!("GPU 内存使用: {:.2} MB", gpu_arrays as f64 / 1024.0 / 1024.0)
    
    # 内存不足时的处理
    if gpu_arrays > 8 * 1024 * 1024 * 1024 {  # 8GB
        println!("GPU 内存不足，建议使用 CPU 或减少批次大小")
    }
}

# 自动内存清理
micro auto_cleanup() {
    # 清理未使用的数组
    ArrayND::gc_collect()
    
    # 释放临时缓存
    ArrayND::clear_cache()
}
```

## 性能监控和调优

### 简单性能测试

```valkyrie
# 测试算法性能
micro benchmark_algorithm<F>(name: &str, f: F) -> Duration 
where F: FnOnce() {
    let start = std::time::Instant::now()
    f()
    let duration = start.elapsed()
    
    println!("{} 耗时: {:.2}ms", name, duration.as_millis())
    duration
}

# 比较不同设备性能
micro compare_devices() {
    let data = ArrayND::random([1000, 1000])
    
    # CPU 计算
    let cpu_time = benchmark_algorithm("CPU 矩阵乘法", || {
        let result = data.matmul(&data)
    })
    
    # GPU 计算
    let gpu_time = benchmark_algorithm("GPU 矩阵乘法", || {
        let data_gpu = data.to_device(&Device::GPU(0))
        let result = data_gpu.matmul(&data_gpu)
    })
    
    println!("GPU 加速比: {:.2}x", cpu_time.as_millis() as f64 / gpu_time.as_millis() as f64)
}```

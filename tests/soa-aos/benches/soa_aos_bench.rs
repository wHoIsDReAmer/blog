use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SIZE: usize = 1_000_000;

// AoS (Array of Structures) - 전통적인 구조체 배열
#[derive(Clone, Copy)]
struct ParticleAoS {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
    mass: f32,
}

// SoA (Structure of Arrays) - 각 필드별로 별도 배열
struct ParticlesSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    vz: Vec<f32>,
    mass: Vec<f32>,
}

// AoSoA (Array of Structures of Arrays) - 구조체 배열의 배열
const CHUNK_SIZE: usize = 8;

#[derive(Clone)]
struct ParticleChunk {
    x: [f32; CHUNK_SIZE],
    y: [f32; CHUNK_SIZE],
    z: [f32; CHUNK_SIZE],
    vx: [f32; CHUNK_SIZE],
    vy: [f32; CHUNK_SIZE],
    vz: [f32; CHUNK_SIZE],
    mass: [f32; CHUNK_SIZE],
    count: usize, // 실제 사용된 요소 수
}

struct ParticlesAoSoA {
    chunks: Vec<ParticleChunk>,
}

impl ParticlesSoA {
    fn new(size: usize) -> Self {
        Self {
            x: vec![0.0; size],
            y: vec![0.0; size],
            z: vec![0.0; size],
            vx: vec![1.0; size],
            vy: vec![1.0; size],
            vz: vec![1.0; size],
            mass: vec![1.0; size],
        }
    }

    fn update_positions(&mut self, dt: f32) {
        for i in 0..self.x.len() {
            self.x[i] += self.vx[i] * dt;
            self.y[i] += self.vy[i] * dt;
            self.z[i] += self.vz[i] * dt;
        }
    }

    fn calculate_kinetic_energy(&self) -> f32 {
        let mut total = 0.0;
        for i in 0..self.x.len() {
            let v_squared = self.vx[i] * self.vx[i] + 
                           self.vy[i] * self.vy[i] + 
                           self.vz[i] * self.vz[i];
            total += 0.5 * self.mass[i] * v_squared;
        }
        total
    }
}

impl ParticlesAoSoA {
    fn new(size: usize) -> Self {
        let num_chunks = (size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let mut chunks = Vec::with_capacity(num_chunks);
        
        for i in 0..num_chunks {
            let remaining = size - i * CHUNK_SIZE;
            let chunk_count = remaining.min(CHUNK_SIZE);
            
            chunks.push(ParticleChunk {
                x: [0.0; CHUNK_SIZE],
                y: [0.0; CHUNK_SIZE],
                z: [0.0; CHUNK_SIZE],
                vx: [1.0; CHUNK_SIZE],
                vy: [1.0; CHUNK_SIZE],
                vz: [1.0; CHUNK_SIZE],
                mass: [1.0; CHUNK_SIZE],
                count: chunk_count,
            });
        }
        
        Self { chunks }
    }

    fn update_positions(&mut self, dt: f32) {
        for chunk in &mut self.chunks {
            for i in 0..chunk.count {
                chunk.x[i] += chunk.vx[i] * dt;
                chunk.y[i] += chunk.vy[i] * dt;
                chunk.z[i] += chunk.vz[i] * dt;
            }
        }
    }

    fn calculate_kinetic_energy(&self) -> f32 {
        let mut total = 0.0;
        for chunk in &self.chunks {
            for i in 0..chunk.count {
                let v_squared = chunk.vx[i] * chunk.vx[i] + 
                               chunk.vy[i] * chunk.vy[i] + 
                               chunk.vz[i] * chunk.vz[i];
                total += 0.5 * chunk.mass[i] * v_squared;
            }
        }
        total
    }
}

// AoS 벤치마크 함수들
fn update_positions_aos(particles: &mut Vec<ParticleAoS>, dt: f32) {
    for particle in particles {
        particle.x += particle.vx * dt;
        particle.y += particle.vy * dt;
        particle.z += particle.vz * dt;
    }
}

fn calculate_kinetic_energy_aos(particles: &Vec<ParticleAoS>) -> f32 {
    let mut total = 0.0;
    for particle in particles {
        let v_squared = particle.vx * particle.vx + 
                       particle.vy * particle.vy + 
                       particle.vz * particle.vz;
        total += 0.5 * particle.mass * v_squared;
    }
    total
}

fn bench_aos_update_positions(c: &mut Criterion) {
    let mut particles = vec![ParticleAoS {
        x: 0.0, y: 0.0, z: 0.0,
        vx: 1.0, vy: 1.0, vz: 1.0,
        mass: 1.0,
    }; SIZE];

    c.bench_function("AoS update positions", |b| {
        b.iter(|| {
            update_positions_aos(black_box(&mut particles), black_box(0.016));
        })
    });
}

fn bench_soa_update_positions(c: &mut Criterion) {
    let mut particles = ParticlesSoA::new(SIZE);

    c.bench_function("SoA update positions", |b| {
        b.iter(|| {
            particles.update_positions(black_box(0.016));
        })
    });
}

fn bench_aosoa_update_positions(c: &mut Criterion) {
    let mut particles = ParticlesAoSoA::new(SIZE);

    c.bench_function("AoSoA update positions", |b| {
        b.iter(|| {
            particles.update_positions(black_box(0.016));
        })
    });
}

fn bench_aos_kinetic_energy(c: &mut Criterion) {
    let particles = vec![ParticleAoS {
        x: 0.0, y: 0.0, z: 0.0,
        vx: 1.0, vy: 1.0, vz: 1.0,
        mass: 1.0,
    }; SIZE];

    c.bench_function("AoS kinetic energy", |b| {
        b.iter(|| {
            black_box(calculate_kinetic_energy_aos(black_box(&particles)));
        })
    });
}

fn bench_soa_kinetic_energy(c: &mut Criterion) {
    let particles = ParticlesSoA::new(SIZE);

    c.bench_function("SoA kinetic energy", |b| {
        b.iter(|| {
            black_box(particles.calculate_kinetic_energy());
        })
    });
}

fn bench_aosoa_kinetic_energy(c: &mut Criterion) {
    let particles = ParticlesAoSoA::new(SIZE);

    c.bench_function("AoSoA kinetic energy", |b| {
        b.iter(|| {
            black_box(particles.calculate_kinetic_energy());
        })
    });
}

// 메모리 접근 패턴 테스트 - 특정 필드만 접근
fn bench_aos_x_coordinate_sum(c: &mut Criterion) {
    let particles = vec![ParticleAoS {
        x: 1.0, y: 2.0, z: 3.0,
        vx: 1.0, vy: 1.0, vz: 1.0,
        mass: 1.0,
    }; SIZE];

    c.bench_function("AoS sum x coordinates", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for particle in black_box(&particles) {
                sum += particle.x;
            }
            black_box(sum)
        })
    });
}

fn bench_soa_x_coordinate_sum(c: &mut Criterion) {
    let mut particles = ParticlesSoA::new(SIZE);
    // x 좌표를 1.0으로 초기화
    for x in &mut particles.x {
        *x = 1.0;
    }

    c.bench_function("SoA sum x coordinates", |b| {
        b.iter(|| {
            let sum: f32 = black_box(&particles.x).iter().sum();
            black_box(sum)
        })
    });
}

fn bench_aosoa_x_coordinate_sum(c: &mut Criterion) {
    let mut particles = ParticlesAoSoA::new(SIZE);
    // x 좌표를 1.0으로 초기화
    for chunk in &mut particles.chunks {
        for i in 0..chunk.count {
            chunk.x[i] = 1.0;
        }
    }

    c.bench_function("AoSoA sum x coordinates", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for chunk in black_box(&particles.chunks) {
                for i in 0..chunk.count {
                    sum += chunk.x[i];
                }
            }
            black_box(sum)
        })
    });
}

criterion_group!(
    position_update_benches,
    bench_aos_update_positions,
    bench_soa_update_positions,
    bench_aosoa_update_positions
);

criterion_group!(
    energy_calculation_benches,
    bench_aos_kinetic_energy,
    bench_soa_kinetic_energy,
    bench_aosoa_kinetic_energy
);

criterion_group!(
    memory_access_benches,
    bench_aos_x_coordinate_sum,
    bench_soa_x_coordinate_sum,
    bench_aosoa_x_coordinate_sum
);

criterion_main!(
    position_update_benches,
    energy_calculation_benches,
    memory_access_benches
);

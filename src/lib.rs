#![cfg_attr(
    all(
        any(target_arch = "arm", target_arch = "aarch64"),
        target_feature = "neon"
    ),
    feature(stdsimd)
)]

pub mod pga3;
pub mod simd;

impl Exp for pga3::Line {
    type Output = pga3::Motor;

    fn exp(self) -> pga3::Motor {
        let det = self.g1[0] * self.g1[0] + self.g1[1] * self.g1[1] + self.g1[2] * self.g1[2];
        if det <= 0.0 {
            return pga3::Motor {
                g0: [1.0, 0.0, 0.0, 0.0].into(),
                g1: [0.0, self.g0[0], self.g0[1], self.g0[2]].into(),
            };
        }
        let a = det.sqrt();
        let c = a.cos();
        let s = a.sin() / a;
        let m = self.g0[0] * self.g1[0] + self.g0[1] * self.g1[1] + self.g0[2] * self.g1[2];
        let t = m / det * (c - s);
        let g0 = simd::Simd32x3::from(s) * self.g1;
        let g1 = simd::Simd32x3::from(s) * self.g0 + simd::Simd32x3::from(t) * self.g1;
        pga3::Motor {
            g0: [c, g0[0], g0[1], g0[2]].into(),
            g1: [s * m, g1[0], g1[1], g1[2]].into(),
        }
    }
}

impl Ln for pga3::Motor {
    type Output = pga3::Line;

    fn ln(self) -> pga3::Line {
        let det = 1.0 - self.g0[0] * self.g0[0];
        if det <= 0.0 {
            return pga3::Line {
                g0: [self.g1[1], self.g1[2], self.g1[3]].into(),
                g1: [0.0, 0.0, 0.0].into(),
            };
        }
        let a = 1.0 / det;
        let b = self.g0[0].acos() * a.sqrt();
        let c = a * self.g1[0] * (1.0 - self.g0[0] * b);
        let g0 = simd::Simd32x4::from(b) * self.g1 + simd::Simd32x4::from(c) * self.g0;
        let g1 = simd::Simd32x4::from(b) * self.g0;
        pga3::Line {
            g0: [g0[1], g0[2], g0[3]].into(),
            g1: [g1[1], g1[2], g1[3]].into(),
        }
    }
}

impl Powf for pga3::Motor {
    type Output = Self;

    fn powf(self, exponent: f32) -> Self {
        (pga3::Scalar { g0: exponent } * self.ln()).exp()
    }
}

impl ScalarPart for pga3::Motor {
    fn scalar_part(self) -> f32 {
        self.g0[0]
    }
}

impl Exp for pga3::IdealLine {
    type Output = pga3::Translator;

    fn exp(self) -> pga3::Translator {
        pga3::Translator {
            g0: [1.0, self.g0[0], self.g0[1], self.g0[2]].into(),
        }
    }
}

impl Ln for pga3::Translator {
    type Output = pga3::IdealLine;

    fn ln(self) -> pga3::IdealLine {
        pga3::IdealLine {
            g0: [
                self.g0[1] / self.g0[0],
                self.g0[2] / self.g0[0],
                self.g0[3] / self.g0[0],
            ]
            .into(),
        }
    }
}

impl Powf for pga3::Translator {
    type Output = Self;

    fn powf(self, exponent: f32) -> Self {
        (pga3::Scalar { g0: exponent } * self.ln()).exp()
    }
}

impl ScalarPart for pga3::Translator {
    fn scalar_part(self) -> f32 {
        self.g0[0]
    }
}

impl Exp for pga3::Branch {
    type Output = pga3::Rotor;

    fn exp(self) -> pga3::Rotor {
        let n = self.magnitude().g0;
        if n == 0.0 {
            pga3::Rotor::one()
        } else {
            let imag = n.sin() / n;
            pga3::Rotor {
                g0: [
                    n.cos(),
                    imag * self.g0[0],
                    imag * self.g0[1],
                    imag * self.g0[2],
                ]
                .into(),
            }
        }
    }
}

impl Ln for pga3::Rotor {
    type Output = pga3::Branch;

    fn ln(self) -> pga3::Branch {
        let n = self.magnitude().g0;
        if n == 0.0 {
            pga3::Branch::zero()
        } else {
            let angle = (self.g0[0] / n).acos();
            let real = angle / angle.sin() / n;
            pga3::Branch {
                g0: [real * self.g0[1], real * self.g0[2], real * self.g0[3]].into(),
            }
        }
    }
}

impl ScalarPart for pga3::Rotor {
    fn scalar_part(self) -> f32 {
        self.g0[0]
    }
}

impl Powf for pga3::Rotor {
    type Output = Self;

    fn powf(self, exponent: f32) -> Self {
        if self.g0[1] == 0.0 && self.g0[2] == 0.0 && self.g0[3] == 0.0 {
            // Rotor is a scalar and thus has no logarithm.
            // Exponentiate just the scalar.
            Self {
                g0: [
                    pga3::Scalar { g0: self.g0[0] }.powf(exponent).g0,
                    0.0,
                    0.0,
                    0.0,
                ]
                .into(),
            }
        } else {
            (pga3::Scalar { g0: exponent } * self.ln()).exp()
        }
    }
}

impl Exp for pga3::Scalar {
    type Output = pga3::Scalar;

    fn exp(self) -> pga3::Scalar {
        pga3::Scalar { g0: self.g0.exp() }
    }
}

impl Ln for pga3::Scalar {
    type Output = pga3::Scalar;

    fn ln(self) -> pga3::Scalar {
        pga3::Scalar { g0: self.g0.ln() }
    }
}

impl Powf for pga3::Scalar {
    type Output = Self;

    fn powf(self, exponent: f32) -> Self {
        pga3::Scalar {
            g0: self.g0.powf(exponent),
        }
    }
}

impl pga3::Scalar {
    pub fn new(x: f32) -> Self {
        Self { g0: x }
    }
}

impl pga3::Point {
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            g0: [x, y, z, 1.0].into(),
        }
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            g0: [x, y, z, w].into(),
        }
    }

    pub fn origin() -> Self {
        Self {
            g0: [0.0, 0.0, 0.0, 1.0].into(),
        }
    }

    pub fn dir(self) -> pga3::Dir {
        let p = self / self.magnitude();
        pga3::Dir {
            g0: [p.g0[0], p.g0[1], p.g0[2]].into(),
        }
    }
}

impl pga3::Origin {
    pub fn new() -> Self {
        Self { g0: 1.0 }
    }
}

impl pga3::Dir {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            g0: [x, y, z].into(),
        }
    }

    pub fn point(self) -> pga3::Point {
        pga3::Point {
            g0: [self.g0[0], self.g0[1], self.g0[2], 1.0].into(),
        }
    }

    pub fn length(self) -> pga3::Scalar {
        self.dual().magnitude()
    }

    pub fn normalize(self) -> Self {
        self / self.length()
    }
}

impl pga3::Branch {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            g0: [x, y, z].into(),
        }
    }
}

impl pga3::Plane {
    pub fn new(x: f32, y: f32, z: f32, distance: f32) -> Self {
        Self {
            g0: [x, y, z, -distance].into(),
        }
    }
}

impl pga3::Flat {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            g0: [x, y, z].into(),
        }
    }
}

impl pga3::Translator {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            g0: [1.0, -x / 2.0, -y / 2.0, -z / 2.0].into(),
        }
    }

    /// Converts a translator to a 4x4 matrix for WebGPU.
    pub fn matrix(&self) -> [pga3::Point; 4] {
        let row = |index: usize| {
            let mut point = pga3::Point::zero();
            point.g0[index] = 1.0;
            let row = self.transformation(point);
            pga3::Point {
                g0: [row.g0[0], row.g0[1], row.g0[2], row.g0[3]].into(),
            }
        };
        [row(0), row(1), row(2), row(3)]
    }
}

impl pga3::Rotor {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            g0: [w, x, y, z].into(),
        }
    }

    pub fn from_angle_axis(angle: f32, axis: pga3::Dir) -> Self {
        let sine = (angle * 0.5).sin();
        Self {
            g0: [
                (0.5 * angle).cos(),
                sine * axis.g0[0],
                sine * axis.g0[1],
                sine * axis.g0[2],
            ]
            .into(),
        }
    }

    /// Converts a rotor to a 4x4 matrix for WebGPU.
    pub fn matrix(&self) -> [pga3::Point; 4] {
        let row = |index: usize| {
            let mut point = pga3::Point::zero();
            point.g0[index] = 1.0;
            let row = self.transformation(point);
            pga3::Point {
                g0: [row.g0[0], row.g0[1], row.g0[2], row.g0[3]].into(),
            }
        };
        [row(0), row(1), row(2), row(3)]
    }
}

impl pga3::Motor {
    /// Converts a motor to a 4x4 matrix for WebGPU.
    pub fn matrix(&self) -> [pga3::Point; 4] {
        let row = |index: usize| {
            let mut point = pga3::Point::zero();
            point.g0[index] = 1.0;
            let row = self.transformation(point);
            pga3::Point {
                g0: [row.g0[0], row.g0[1], row.g0[2], row.g0[3]].into(),
            }
        };
        [row(0), row(1), row(2), row(3)]
    }
}

/// Distance between two geometric objects
pub fn distance<T, P, R, S>(a: T, b: P) -> S
where
    T: Copy + Magnitude<Output = S> + RegressiveProduct<P, Output = R>,
    P: Copy + Magnitude<Output = S>,
    R: Magnitude<Output = S>,
    S: std::ops::Mul<S, Output = S> + std::ops::Div<S, Output = S>,
{
    (a.regressive_product(b)).magnitude() / (a.magnitude() * b.magnitude())
}

/// Length of ideal part as scalar.
///
/// Also called ideal norm.
pub fn ideal_magnitude<T, S>(v: T) -> S
where
    T: Dual<Output = T>,
    T: Magnitude<Output = S>,
{
    v.dual().magnitude()
}

/// Projects `a` into `b`.
pub fn project<A, B, I, C>(a: A, b: B) -> C
where
    A: RightContraction<B, Output = I>,
    B: Copy + Inverse<Output = B>,
    I: OuterProduct<B, Output = C>,
{
    a.right_contraction(b.inverse()).outer_product(b)
}

/// Antiprojects `a` onto `b`.
pub fn anti_project<A, B, I, C>(a: A, b: B) -> C
where
    A: LeftContraction<B, Output = I>,
    B: Copy + Inverse<Output = B>,
    I: LeftContraction<B, Output = C>,
{
    a.left_contraction(b.inverse()).left_contraction(b)
}

/// Generates a motion from `a` to `b`.
pub fn motion<A, B, M, S>(a: A, b: B) -> M
where
    A: Copy + Magnitude<Output = S> + std::ops::Div<S, Output = A>,
    B: Copy + Magnitude<Output = S> + std::ops::Div<S, Output = B> + std::ops::Mul<A, Output = M>,
    M: Copy + ScalarPart + std::ops::Neg<Output = M> + Powf<Output = M>,
{
    ((b / b.magnitude()) * (a / a.magnitude()))
        .constrain()
        .sqrt()
}

/// Interpolate from `a` to `b`.
pub fn interpolate<A, B, M>(a: A, b: B, interpolant: f32) -> M
where
    A: Copy + Reversal<Output = A>,
    B: std::ops::Mul<A, Output = M>,
    M: ScalarPart + Powf<Output = M> + std::ops::Mul<A, Output = M>,
{
    (b * a.reversal()).constrain().powf(interpolant) * a
}

/// All elements set to `0.0`
pub trait Zero {
    fn zero() -> Self;
}

/// All elements set to `0.0`, except for the scalar, which is set to `1.0`
pub trait One {
    fn one() -> Self;
}

/// Element order reversed
pub trait Dual {
    type Output;
    fn dual(self) -> Self::Output;
}

/// Negates elements with `grade % 2 == 1`
///
/// Also called main involution
pub trait Automorphism {
    type Output;
    fn automorphism(self) -> Self::Output;
}

/// Negates elements with `grade % 4 >= 2`
///
/// Also called transpose
pub trait Reversal {
    type Output;
    fn reversal(self) -> Self::Output;
}

/// Negates elements with `(grade + 3) % 4 < 2`
pub trait Conjugation {
    type Output;
    fn conjugation(self) -> Self::Output;
}

/// General multi vector multiplication
pub trait GeometricProduct<T> {
    type Output;
    fn geometric_product(self, other: T) -> Self::Output;
}

/// Dual of the geometric product grade filtered by `t == r + s`
///
/// Also called join
pub trait RegressiveProduct<T> {
    type Output;
    fn regressive_product(self, other: T) -> Self::Output;
}

/// Geometric product grade filtered by `t == r + s`
///
/// Also called meet or exterior product
pub trait OuterProduct<T> {
    type Output;
    fn outer_product(self, other: T) -> Self::Output;
}

/// Geometric product grade filtered by `t == (r - s).abs()`
///
/// Also called fat dot product
pub trait InnerProduct<T> {
    type Output;
    fn inner_product(self, other: T) -> Self::Output;
}

/// Geometric product grade filtered by `t == s - r`
pub trait LeftContraction<T> {
    type Output;
    fn left_contraction(self, other: T) -> Self::Output;
}

/// Geometric product grade filtered by `t == r - s`
pub trait RightContraction<T> {
    type Output;
    fn right_contraction(self, other: T) -> Self::Output;
}

/// Geometric product grade filtered by `t == 0`
pub trait ScalarProduct<T> {
    type Output;
    fn scalar_product(self, other: T) -> Self::Output;
}

/// `self * other * self.reversion()`
///
/// Also called sandwich product
pub trait Transformation<T> {
    type Output;
    fn transformation(self, other: T) -> Self::Output;
}

/// Square of the magnitude
pub trait SquaredMagnitude {
    type Output;
    fn squared_magnitude(self) -> Self::Output;
}

/// Length as scalar
///
/// Also called amplitude, absolute value or norm
pub trait Magnitude {
    type Output;
    fn magnitude(self) -> Self::Output;
}

/// Direction without magnitude (set to scalar `-1.0` or `1.0`)
///
/// Also called sign or normalize
pub trait Signum {
    type Output;
    fn signum(self) -> Self::Output;
}

/// Raises a number to the scalar power of `-1.0`
pub trait Inverse {
    type Output;
    fn inverse(self) -> Self::Output;
}

/// The natural logarithm
pub trait Ln {
    type Output;
    fn ln(self) -> Self::Output;
}

/// The exponential function
pub trait Exp {
    type Output;
    fn exp(self) -> Self::Output;
}

/// Raises a number to an integer scalar power
pub trait Powi {
    type Output;
    fn powi(self, exponent: isize) -> Self::Output;
}

/// Raises a number to an floating point scalar power
pub trait Powf
where
    Self: std::marker::Sized,
{
    type Output;
    fn powf(self, exponent: f32) -> Self::Output;

    fn sqrt(self) -> Self::Output {
        self.powf(0.5)
    }
}

/// Extracts the scalar part of a multivector.
pub trait ScalarPart
where
    Self: Copy + std::ops::Neg<Output = Self>,
{
    fn scalar_part(self) -> f32;

    /// Constrain the motion to traverse the shortest arc.
    fn constrain(self) -> Self {
        if self.scalar_part() >= 0.0 {
            self
        } else {
            -self
        }
    }
}

use std::ops;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn mul(self, x: f64) -> Self {
        Self {
            x: (self.x as f64 * x) as i64,
            y: (self.y as f64 * x) as i64,
        }
    }
}
impl ops::Add for Point {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for Point {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Vec2
#[derive(Clone, Debug, Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: ops::Add<Output = T>> ops::Add for Vec2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Vec3
#[derive(Clone, Debug, Copy)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: ops::Add<Output = T>> ops::Add for Vec3<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec3<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T> ops::Mul for Vec3<T>
where
    T: ops::Mul<Output = T> + ops::Add<Output = T>,
{
    type Output = T;
    fn mul(self, other: Self) -> Self::Output {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl<T> Vec3<T>
where
    T: ops::Mul<Output = T> + Copy,
{
    pub fn scale(self, scale: T) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
        }
    }
}

impl<T> Vec3<T>
where
    T: ops::Mul<Output = T> + ops::Add<Output = T> + Into<f64> + From<f64> + Copy,
{
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z)
            .into()
            .sqrt()
    }

    pub fn normalize(self) -> Self {
        let n = self.norm();
        self.scale((1f64 / n).into())
    }
}

impl<T> ops::BitXor for Vec3<T>
where
    T: ops::Mul<Output = T> + ops::Sub<Output = T> + Copy,
{
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

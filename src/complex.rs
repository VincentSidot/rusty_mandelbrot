#[derive(Debug, Clone, Copy)]
pub struct Complex<T> {
    pub re: T,
    pub im: T,
}

#[allow(dead_code)]
impl<T> Complex<T> {
    pub fn new(re: T, im: T) -> Complex<T> {
        Complex { re, im }
    }

    pub fn im(&self) -> &T {
        &self.im
    }

    pub fn re(&self) -> &T {
        &self.re
    }

    pub fn im_mut(&mut self) -> &mut T {
        &mut self.im
    }

    pub fn re_mut(&mut self) -> &mut T {
        &mut self.re
    }

    pub fn norm(&self) -> T
    where
        T: Copy + std::ops::Add<T, Output = T> + std::ops::Mul<T, Output = T>,
    {
        self.re * self.re + self.im * self.im
    }

    pub fn conj(&self) -> Complex<T>
    where
        T: Copy + std::ops::Neg<Output = T>,
    {
        Complex {
            re: self.re,
            im: -self.im,
        }
    }
}

impl Complex<f64> {
    const I: Complex<f64> = Complex { re: 0.0, im: 1.0 };

    pub fn exp(&self) -> Self {
        let exp_re = self.re.exp();
        Self {
            re: exp_re * self.im.cos(),
            im: exp_re * self.im.sin(),
        }
    }

    pub fn cos(&self) -> Self {
        (Self::I * *self).exp() + (-Self::I * *self).exp() / 2.0
    }
}

pub mod op {
    use super::Complex;
    use std::ops::*;

    impl<T: AddAssign<T>> AddAssign for Complex<T> {
        fn add_assign(&mut self, rhs: Complex<T>) {
            self.re += rhs.re;
            self.im += rhs.im;
        }
    }

    impl<T: SubAssign<T>> SubAssign for Complex<T> {
        fn sub_assign(&mut self, rhs: Complex<T>) {
            self.re -= rhs.re;
            self.im -= rhs.im;
        }
    }

    impl<T: Add<T, Output = T>> Add for Complex<T> {
        type Output = Complex<T>;
        fn add(self, rhs: Complex<T>) -> Complex<T> {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }

    impl<T: Sub<T, Output = T>> Sub for Complex<T> {
        type Output = Complex<T>;
        fn sub(self, rhs: Complex<T>) -> Complex<T> {
            Complex {
                re: self.re - rhs.re,
                im: self.im - rhs.im,
            }
        }
    }

    impl<T: Mul<T, Output = T> + Add<T, Output = T> + Sub<T, Output = T> + Copy> Mul for Complex<T> {
        type Output = Complex<T>;
        fn mul(self, rhs: Complex<T>) -> Complex<T> {
            Complex {
                re: self.re * rhs.re - self.im * rhs.im,
                im: self.re * rhs.im + self.im * rhs.re,
            }
        }
    }

    impl<T: Mul<T, Output = T> + Copy> Mul<T> for Complex<T> {
        type Output = Complex<T>;
        fn mul(self, rhs: T) -> Complex<T> {
            Complex {
                re: self.re * rhs,
                im: self.im * rhs,
            }
        }
    }

    impl<T: Div<T, Output = T> + Copy> Div<T> for Complex<T> {
        type Output = Complex<T>;
        fn div(self, rhs: T) -> Complex<T> {
            Complex {
                re: self.re / rhs,
                im: self.im / rhs,
            }
        }
    }

    impl<
            T: Mul<T, Output = T>
                + Add<T, Output = T>
                + Sub<T, Output = T>
                + Div<T, Output = T>
                + Copy,
        > Div for Complex<T>
    {
        type Output = Complex<T>;
        fn div(self, rhs: Complex<T>) -> Complex<T> {
            let norm = rhs.norm();
            Complex {
                re: (self.re * rhs.re + self.im * rhs.im) / norm,
                im: (self.im * rhs.re - self.re * rhs.im) / norm,
            }
        }
    }

    impl<T: Neg<Output = T>> Neg for Complex<T> {
        type Output = Complex<T>;
        fn neg(self) -> Complex<T> {
            Complex {
                re: -self.re,
                im: -self.im,
            }
        }
    }
}

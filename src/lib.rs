#![deny(missing_docs)]

//! A library for higher order functional programming with homotopy maps to construct 3D geometry.
//!
//! ### What is a homotopy map?
//!
//! A [homotopy](https://en.wikipedia.org/wiki/Homotopy) is a continuous
//! deformation between two functions.
//! Think about combining two functions `f` and `g` with a parameter in the range
//! between 0 and 1 such that setting the parameter to 0 gives you `f` and
//! setting it to 1 gives you `g`.
//! With other words, it lets you interpolate smoothly between functions.
//!
//! This library uses a simplified homotopy version designed for constructing 3D geometry:
//!
//! ```rust
//! /// A function of type `1d -> 3d`.
//! pub type Fn1<T> = Arc<Fn(T) -> [T; 3] + Sync + Send>;
//! /// A function of type `2d -> 3d`.
//! pub type Fn2<T> = Arc<Fn([T; 2]) -> [T; 3] + Sync + Send>;
//! /// A function of type `3d -> 3d`.
//! pub type Fn3<T> = Arc<Fn([T; 3]) -> [T; 3] + Sync + Send>;
//! ```
//!
//! In this library, these functions are called *homotopy maps* and usually
//! satisfies these properties:
//!
//! - All inputs are assumed to be normalized, starting at 0 and ending at 1.
//!   This means that `Fn1` forms a curved line, `Fn2` forms a curved quad,
//!   and `Fn3` forms a curved cube.
//! - The `Arc` smart pointer makes it possible to clone closures.
//! - The `Sync` and `Send` constraints makes it easier to program with multiple threads.
//! - Basic geometric shapes are continuous within the range from 0 to 1.
//!
//! A curved cube does not mean it need to look like a cube.
//! Actually, you can create a variety of shapes that do not look like cubes at all,
//! e.g. a sphere.
//! What is meant by a "curved cube" is that there are 3 parameters between 0 and 1
//! controlling the generation of points.
//! If you used an identity map, you would get a cube shape.
//! The transformation to other shapes is the reason it is called a "curved cube".
//!
//! ### Motivation
//!
//! Constructing 3D geometry is an iterative process where the final design/need
//! can be quite different from the first draft.
//! In game engines there are additional needs like generating multiple models of various
//! detail or adjusting models depending on the capacity of the target platform.
//! This makes it desirable to have some tools where one can work with an idea without
//! getting slowed down by a lot of technical details.
//!
//! Homotopy maps have the property that the geometry can be constructed by need,
//! without any additional instructions.
//! This makes it a suitable candidate for combining them with higher order functional programming.
//! Functions give an accurate representation while at the same time being lazy,
//! such that one can e.g. intersect a curved cube to get a curved quad.
//!
//! This library is an experiment to see how homotopy maps and higher order functional programming
//! can be used to iterate on design.
//! Function names are very short to provide good ergonomics.

extern crate vecmath;

pub use vecmath::vec3_add as add3;
pub use vecmath::vec2_add as add2;
pub use vecmath::vec3_sub as sub3;
pub use vecmath::vec2_sub as sub2;
pub use vecmath::vec3_len as len3;
pub use vecmath::vec2_len as len2;
pub use vecmath::vec3_scale as scale3;
pub use vecmath::vec2_scale as scale2;
pub use vecmath::vec2_cast as cast2;
pub use vecmath::vec3_cast as cast3;
pub use vecmath::traits::*;

use std::sync::Arc;

/// A function of type `1d -> 3d`.
pub type Fn1<T> = Arc<Fn(T) -> [T; 3] + Sync + Send>;
/// A function of type `2d -> 3d`.
pub type Fn2<T> = Arc<Fn([T; 2]) -> [T; 3] + Sync + Send>;
/// A function of type `3d -> 3d`.
pub type Fn3<T> = Arc<Fn([T; 3]) -> [T; 3] + Sync + Send>;

/// Returns a linear function.
pub fn lin<T: Float>(a: [T; 3], b: [T; 3]) -> Fn1<T> {
    return Arc::new(move |t| add3(a, scale3(sub3(b, a), t)))
}

/// Creates a linear interpolation between two functions.
pub fn lin2<T: Float>(a: Fn1<T>, b: Fn1<T>) -> Fn1<T> {
    return Arc::new(move |t| {
        add3(scale3(a(t), <T as One>::one() - t), scale3(b(t), t))
    })
}

/// Quadratic bezier curve.
pub fn qbez<T: Float>(a: [T; 3], b: [T; 3], c: [T; 3]) -> Fn1<T> {
    lin2(lin(a, b), lin(b, c))
}

/// Cubic bezier curve.
pub fn cbez<T: Float>(
    a: [T; 3],
    b: [T; 3],
    c: [T; 3],
    d: [T; 3],
) -> Fn1<T> {
    lin2(lin(a, b), lin(c, d))
}

/// Constructs a curved quad by smoothing between boundary functions.
pub fn cquad<T: Float>(
    smooth: T,
    ab: Fn1<T>,
    cd: Fn1<T>,
    ac: Fn1<T>,
    bd: Fn1<T>
) -> Fn2<T>
    where f64: Cast<T>
{
    let _1: T = One::one();
    let _0: T = Zero::zero();
    let _05: T = 0.5.cast();
    let _4: T = 4.0.cast();
    return Arc::new(move |t| {
        let abx = ab(t[1]);
        let cdx = cd(t[1]);
        let acx = ac(t[0]);
        let bdx = bd(t[0]);

        let w0 = _4 * (t[0] - _05) * (t[0] - _05) + smooth;
        let w1 = _4 * (t[1] - _05) * (t[1] - _05) + smooth;
        // Normalize weights.
        let (w0, w1) = (w0 / (w0 + w1), w1 / (w0 + w1));

        let a = add3(abx, scale3(sub3(cdx, abx), t[0]));
        let b = add3(acx, scale3(sub3(bdx, acx), t[1]));
        if w0 == _1 {a}
        else if w1 == _1 {b}
        else if (w0 + w1) == _0 {
            scale3(add3(a, b), _05)
        }
        else {
            add3(scale3(a, w0), scale3(b, w1))
        }
    })
}

/// Concatenates two `1d -> 3d` functions returning a new function.
///
/// The input to the new function is normalized.
pub fn con<T: Float>(w: T, a: Fn1<T>, b: Fn1<T>) -> Fn1<T> {
    return Arc::new(move |t| {
        if t < w {a(t / w)}
        else {b((t - w) / (<T as One>::one() - w))}
    })
}

/// Concatenates two `2d -> 3d` functions at x-weight.
pub fn conx2<T: Float>(wx: T, a: Fn2<T>, b: Fn2<T>) -> Fn2<T> {
    return Arc::new(move |t| {
        if t[0] < wx {a([t[0] / wx, t[1]])}
        else {b(([(t[0] - wx) / (<T as One>::one() - wx), t[1]]))}
    })
}

/// Concatenates two `2d -> 3d` functions at y-weight.
pub fn cony2<T: Float>(wy: T, a: Fn2<T>, b: Fn2<T>) -> Fn2<T> {
    return Arc::new(move |t| {
        if t[1] < wy {a([t[0], t[1] / wy])}
        else {b([t[0], (t[1] - wy) / (<T as One>::one() - wy)])}
    })
}

/// Concatenates two `3d -> 3d` functions at x-weight.
pub fn conx3<T: Float>(wx: T, a: Fn3<T>, b: Fn3<T>) -> Fn3<T> {
    return Arc::new(move |t| {
        if t[0] < wx {a([t[0] / wx, t[1], t[2]])}
        else {b(([(t[0] - wx) / (<T as One>::one() - wx), t[1], t[2]]))}
    })
}

/// Concates two `3d -> 3d` functions at y-weight.
pub fn cony3<T: Float>(wy: T, a: Fn3<T>, b: Fn3<T>) -> Fn3<T> {
    return Arc::new(move |t| {
        if t[1] < wy {a([t[0], t[1] / wy, t[2]])}
        else {b([t[0], (t[1] - wy) / (<T as One>::one() - wy), t[2]])}
    })
}

/// Concates two `3d -> 3d` functions at z-weight.
pub fn conz3<T: Float>(wz: T, a: Fn3<T>, b: Fn3<T>) -> Fn3<T> {
    return Arc::new(move |t| {
        if t[2] < wz {a([t[0], t[1], t[2] / wz])}
        else {b([t[0], t[1], (t[2] - wz) / (<T as One>::one() - wz)])}
    })
}

/// Mirror shape `1d -> 3d` around yz-plane at x coordinate.
pub fn mx<T: 'static, U: Float>(
    x: U,
    a: Arc<Fn(T) -> [U; 3] + Sync + Send>
) -> Arc<Fn(T) -> [U; 3] + Sync + Send>
    where f64: Cast<U>
{
    return Arc::new(move |t| {
        let pos = a(t);
        [2.0.cast() * x - pos[0], pos[1], pos[2]]
    })
}

/// Mirror shape `1d -> 3d` around xz-plane at y coordinate.
pub fn my<T: 'static, U: Float>(
    y: U,
    a: Arc<Fn(T) -> [U; 3] + Sync + Send>
) -> Arc<Fn(T) -> [U; 3] + Sync + Send>
    where f64: Cast<U>
{
    return Arc::new(move |t| {
        let pos = a(t);
        [pos[0], 2.0.cast() * y - pos[1], pos[2]]
    })
}

/// Mirror shape `1d -> 3d` around xy-plane at z coordinate.
pub fn mz<T: 'static, U: Float>(
    z: U,
    a: Arc<Fn(T) -> [U; 3] + Sync + Send>
) -> Arc<Fn(T) -> [U; 3] + Sync + Send>
    where f64: Cast<U>
{
    return Arc::new(move |t| {
        let pos = a(t);
        [pos[0], pos[1], 2.0.cast() * z - pos[2]]
    })
}

/// Bake mirror `2d -> 3d` around yz-plane at x coordinate.
pub fn mirx2<T: Float>(x: T, a: Fn2<T>) -> Fn2<T>
    where f64: Cast<T>
{
    conx2(0.5.cast(), a.clone(), mx(x, a))
}

/// Bake mirror `2d -> 3d` around xz-plane at y coordinate.
pub fn miry2<T: Float>(y: T, a: Fn2<T>) -> Fn2<T>
    where f64: Cast<T>
{
    cony2(0.5.cast(), a.clone(), my(y, a))
}

/// Bake mirror `3d -> 3d` around yz-plane at x coordinate.
pub fn mirx3<T: Float>(x: T, a: Fn3<T>) -> Fn3<T>
    where f64: Cast<T>
{
    conx3(0.5.cast(), a.clone(), mx(x, a))
}

/// Bake mirror `3d -> 3d` around xz-plane at y coordinate.
pub fn miry3<T: Float>(y: T, a: Fn3<T>) -> Fn3<T>
    where f64: Cast<T>
{
    cony3(0.5.cast(), a.clone(), my(y, a))
}

/// Bake mirror `3d -> 3d` around xy-plane at z coordinate.
pub fn mirz3<T: Float>(z: T, a: Fn3<T>) -> Fn3<T>
    where f64: Cast<T>
{
    conz3(0.5.cast(), a.clone(), mz(z, a))
}

/// Reverses input direction.
pub fn rev<T: Float>(a: Fn1<T>) -> Fn1<T> {
    seg1([One::one(), Zero::zero()], a)
}

/// Offsets `3d -> 3d` at position.
pub fn off<T: 'static, U: Float>(
    pos: [U; 3],
    a: Arc<Fn(T) -> [U; 3] + Sync + Send>
) -> Arc<Fn(T) -> [U; 3] + Sync + Send> {
    return Arc::new(move |t| add3(a(t), pos))
}

/// Gets the contour line of a curved quad.
///
/// ```ignore
/// 0.0-0.25: [0.0, 0.0] -> [1.0, 0.0]
/// 0.25-0.5: [1.0, 0.0] -> [1.0, 1.0]
/// 0.5-0.75: [1.0, 1.0] -> [0.0, 1.0]
/// 0.75-1.0: [0.0, 1.0] -> [0.0, 0.0]
/// ```
pub fn contour<T: Float>(a: Fn2<T>) -> Fn1<T>
    where f64: Cast<T>
{
    let _025: T = 0.25.cast();
    let _4: T = 4.0.cast();
    let _0: T = 0.0.cast();
    let _05: T = 0.5.cast();
    let _1: T = 1.0.cast();
    let _075 = 0.75.cast();
    return Arc::new(move |t| {
        if t < _025 {a([_4 * t, _0])}
        else if t < _05 {a([_1, _4 * (t - _025)])}
        else if t < _075 {a([_1 - _4 * (t - _05), _1])}
        else {a([_0, _1 - _4 * (t - _075)])}
    })
}

/// Adds a margin to input of a `1d -> 3d` function.
pub fn margin1<T: Float>(m: T, a: Fn1<T>) -> Fn1<T>
    where f64: Cast<T>
{
    let _1 = 1.0.cast();
    let _2 = 2.0.cast();
    let s = _1 / (_1 + _2 * m);
    return Arc::new(move |t| a((t + m) * s))
}

/// Adds a margin to input of a `2d -> 3d` function.
pub fn margin2<T: Float>(m: T, a: Fn2<T>) -> Fn2<T>
    where f64: Cast<T>
{
    let _1 = 1.0.cast();
    let _2 = 2.0.cast();
    let s = _1 / (_1 + _2 * m);
    return Arc::new(move |t| a([(t[0] + m) * s, (t[1] + m) * s]))
}

/// Adds a margin to input of a `3d -> 3d` function.
pub fn margin3<T: Float>(m: T, a: Fn3<T>) -> Fn3<T>
    where f64: Cast<T>
{
    let _1 = 1.0.cast();
    let _2 = 2.0.cast();
    let s = _1 / (_1 + _2 * m);
    return Arc::new(move |t| a([(t[0] + m) * s, (t[1] + m) * s, (t[2] + m) * s]))
}

/// Creates a circle located at a center and with a radius.
///
/// The first input argument is the angle starting at 0,
/// rotating 360 degrees around the center endering at 1.
/// The second input argument is the radius starting at 0 ending at 1.
///
/// The circle is flat along the z axis.
pub fn circle<T: Float>(center: [T; 3], radius: T) -> Fn2<T>
    where f64: Cast<T>
{
    let two_pi = 6.283185307179586.cast();
    return Arc::new(move |t| {
        let angle = t[0] * two_pi;
        [
            center[0] + radius * t[1] * angle.cos(),
            center[1] + radius * t[1] * angle.sin(),
            center[2]
        ]
    })
}

/// Creates a sphere located at a center and with a radius.
///
/// The two first arguments are angles, the third is radius.
/// The first input argument controls rotation around the z axis.
/// The second input argument starts at the top of the sphere
/// and moves down to the bottom of the sphere.
pub fn sphere<T: Float>(center: [T; 3], radius: T) -> Fn3<T>
    where f64: Cast<T>
{
    let two_pi = 6.283185307179586.cast();
    let _1 = 1.0.cast();
    let _2 = 2.0.cast();
    return Arc::new(move |t| {
        let angle0 = t[0] * two_pi;
        let tx = _2 * t[1] - _1;
        let rad = radius * (_1 - tx * tx).sqrt();
        [
            center[0] + rad * t[2] * angle0.cos(),
            center[1] + rad * t[2] * angle0.sin(),
            center[2] - radius + _2 * radius * t[1],
        ]
    })
}

/// Intersects a curved quad at x-line.
pub fn x2<T: Float>(x: T, a: Fn2<T>) -> Fn1<T>
    where f64: Cast<T>
{
    return Arc::new(move |t| a([x, t]))
}

/// Intersects a curved quad at y-line.
pub fn y2<T: Float>(y: T, a: Fn2<T>) -> Fn1<T> {
    return Arc::new(move |t| a([t, y]))
}

/// Intersects a curved cube at x-plane.
pub fn x3<T: Float>(x: T, a: Fn3<T>) -> Fn2<T> {
    return Arc::new(move |t| a([x, t[0], t[1]]))
}

/// Intersects a curved cube at y-plane.
pub fn y3<T: Float>(y: T, a: Fn3<T>) -> Fn2<T> {
    return Arc::new(move |t| a([t[0], y, t[1]]))
}

/// Intersects a curved cube at z-plane.
pub fn z3<T: Float>(z: T, a: Fn3<T>) -> Fn2<T> {
    return Arc::new(move |t| a([t[0], t[1], z]))
}

/// Extends a 1d shape into 2d by adding a
/// vector to the result generated by a 1d shape.
pub fn ext1<T: Float>(a: Fn1<T>, b: Fn1<T>) -> Fn2<T> {
    return Arc::new(move |t| add3(a(t[0]), b(t[1])))
}

/// Extends a 2d shape into 3d by adding
/// a vector to the result generated by a 1d shape.
pub fn ext2<T: Float>(a: Fn1<T>, b: Fn2<T>) -> Fn3<T> {
    return Arc::new(move |t| add3(a(t[0]), b([t[1], t[2]])))
}

/// Uses a range to pick a segment of a curve.
pub fn seg1<T: Float>(range: [T; 2], a: Fn1<T>) -> Fn1<T> {
    return Arc::new(move |t| a(range[0] + (range[1] - range[0]) * t))
}

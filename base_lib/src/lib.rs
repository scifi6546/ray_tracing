mod voxel;

use cgmath::{InnerSpace, MetricSpace, Point3, Vector3};
use rand::prelude::*;
use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

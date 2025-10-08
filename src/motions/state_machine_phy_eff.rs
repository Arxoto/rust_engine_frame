use std::f64;

use crate::motions::abstracts::player_input::PlayerOperation;

/// 将当前值 `current` 向目标值 `target` 移动最多 `step` 的距离，返回移动后的新值。
/// 如果 `current` 已经达到或超过 `target` ，则返回 `target`
pub fn move_toward(current: f64, target: f64, step: f64) -> f64 {
    // 判断移动方向并计算新值
    if current < target {
        // 向右移动（增加值），但不能超过目标值
        (current + step).min(target)
    } else if current > target {
        // 向左移动（减少值），但不能低于目标值
        (current - step).max(target)
    } else {
        target
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhyAttribute {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl PhyAttribute {
    pub fn velocity_eff(&mut self, delta: f64, eff: PhyEff) {
        self.x = move_toward(self.x, eff.x_velocity, delta * eff.x_acceleration);
        self.y = move_toward(self.y, eff.y_velocity, delta * eff.y_acceleration);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PhyEff {
    pub(crate) x_velocity: f64,
    pub(crate) x_acceleration: f64,
    pub(crate) y_velocity: f64,
    pub(crate) y_acceleration: f64,
}

/// 只有单元测试中可使用 Default
#[derive(Clone, Debug, Default)]
pub struct MotionData {
    // x
    /// 奔跑
    pub(crate) run_x_velocity: f64,
    /// 奔跑
    pub(crate) run_x_resistance: f64,
    /// 奔跑
    pub(crate) run_x_acceleration: f64,

    /// 跳跃下落 与奔跑速度一致即可 建议比跳跃速度略小（跳跃曲线优雅点）
    pub(crate) air_x_velocity: f64,
    /// 跳跃下落 较大时落点可控 提升输入精确度
    pub(crate) air_x_resistance: f64,
    /// 跳跃下落 较大时更灵活
    pub(crate) air_x_acceleration: f64,

    // /// 滑翔飞行 应稍大 体现出飞行的感觉
    // pub(crate) fly_x_velocity: f64,
    // /// 滑翔飞行 应较小 空中滑行惯性大
    // pub(crate) fly_x_resistance: f64,
    // /// 滑翔飞行 应较小 空中转向困难
    // pub(crate) fly_x_acceleration: f64,

    // y
    /// 重力加速度（正常情况下）
    pub(crate) gravity: f64,
    /// 最大下落速度（防止过大） 一般取跳跃速度的两倍
    pub(crate) fall_velocity: f64,

    /// 重力加速度（跳跃时略小 `g' = g * 0.618` ），保证跳跃曲线先缓后急
    pub(crate) jump_gravity: f64,
    /// 跳跃速度
    ///
    /// 最佳实践：跳跃高度最低应该为一格左右 `h = v ** 2 /(g * 2)` `v=sqrt(2 * g * h)`
    ///
    /// 由下推导可推出单元格的高度 `h = 200*200 / 2 / (1000 * 0.618) = 32` （跳跃时重力加速度有个弹性系数）
    pub(crate) jump_velocity: f64,
    /// 跳得更高（开头一段时间跳跃速度不减）
    ///
    /// 最佳实践：跳跃高度最高为三格 保证 `t = v / g` 即可，且时长最好小于 0.2s 否则视觉效果不佳
    ///
    /// 因此可确定起跳速度 `v = g * 0.2` 参考 Godot `g=980px/s^2` 带入 `g=1000; v=200;`
    // pub(crate) jump_higher_time: f64,

    /// 攀爬时的下落速度 可取 `g * (1-0.618)`
    ///
    /// 同时认为攀爬时摩擦力较大 应更为可控 因此可无视加速度直接修改速度
    pub(crate) climb_velocity: f64,
}

impl PhyEff {
    /// 强行静止
    pub fn create_stop(_data: &MotionData, _direction: f64) -> PhyEff {
        PhyEff {
            x_velocity: 0.0,
            x_acceleration: f64::INFINITY,
            y_velocity: 0.0,
            y_acceleration: f64::INFINITY,
        }
    }

    /// 奔跑 传入方向（手柄能控制最大速度，但是摇杆左右为移动、上下为攻击方向，可能导致操作不顺，后期可换成 bool ）
    pub fn create_run(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            x_velocity: direction * data.run_x_velocity,
            x_acceleration: if direction.op_active() {
                data.run_x_acceleration
            } else {
                data.run_x_resistance
            },
            y_velocity: 0.0,
            y_acceleration: f64::INFINITY, // 瞬间将竖直速度置为零 其实使用正常下落速度也可以 可能会减少碰撞运算？
        }
    }

    /// 空中水平移动，用作默认值
    fn create_air_move(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            x_velocity: direction * data.air_x_velocity,
            x_acceleration: if direction.op_active() {
                data.air_x_acceleration
            } else {
                data.air_x_resistance
            },
            y_velocity: 0.0,
            y_acceleration: f64::INFINITY,
        }
    }

    /// 正常下落
    pub fn create_falling(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            y_velocity: data.fall_velocity,
            y_acceleration: data.gravity,
            ..Self::create_air_move(data, direction)
        }
    }

    /// 跳跃上升时的重力加速度较为特殊
    pub fn create_jumping(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            y_velocity: data.fall_velocity,
            y_acceleration: data.jump_gravity,
            ..Self::create_air_move(data, direction)
        }
    }

    /// 跳跃 瞬间加速
    pub fn create_jump(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            y_velocity: data.jump_velocity,
            y_acceleration: f64::INFINITY,
            ..Self::create_air_move(data, direction)
        }
    }

    /// 攀爬下滑
    pub fn create_climb(data: &MotionData, direction: f64) -> PhyEff {
        PhyEff {
            y_velocity: data.climb_velocity,
            y_acceleration: f64::INFINITY,
            ..Self::create_air_move(data, direction)
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_move_toward() {
        assert_eq!(move_toward(0.0, 1.0, 0.9), 0.9);
        assert_eq!(move_toward(0.0, 1.0, 1.0), 1.0);
        assert_eq!(move_toward(0.0, 1.0, 1.9), 1.0);
        assert_eq!(move_toward(-1.0, 1.0, 1.8), 0.8);
        assert_eq!(move_toward(5.0, 0.0, 1.0), 4.0);
        assert_eq!(move_toward(5.0, -1.0, 10.0), -1.0);
        assert_eq!(move_toward(-1.0, -10.0, 1.0), -2.0);

        // 验证精度
        assert_ne!(0.1 + 0.2, 0.3);
        assert!(0.1 + 0.2 > 0.3);
        assert_eq!(move_toward(0.1, 0.3, 0.2), 0.3);

        // 远离
        assert_eq!(move_toward(0.0, 1.0, -0.5), -0.5);
    }

    fn new_data() -> MotionData {
        MotionData {
            run_x_velocity: 200.0,
            run_x_resistance: 4000.0,
            run_x_acceleration: 2000.0,
            air_x_velocity: 200.0,
            air_x_resistance: 1600.0,
            air_x_acceleration: 1600.0,
            // fly_x_velocity: 260.0,
            // fly_x_resistance: 100.0,
            // fly_x_acceleration: 100.0,
            gravity: 980.0,
            fall_velocity: 400.0,
            jump_gravity: 618.0,
            jump_velocity: -200.0,
            // jump_higher_time: 0.2,
            climb_velocity: 80.0,
        }
    }

    #[test]
    fn test_jump_immediately() {
        let mut current = PhyAttribute { x: 0.0, y: 400.0 };
        let phy_eff = PhyEff::create_jump(&new_data(), 1.0);
        current.velocity_eff(0.01, phy_eff);
        assert_eq!(current.y, -200.0);
    }

    // todo test for each create
}

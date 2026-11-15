//! 2048 纯棋盘逻辑(无 UI 依赖,全单测)。

pub type Grid = [[u32; 4]; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn all() -> [Dir; 4] {
        [Dir::Up, Dir::Down, Dir::Left, Dir::Right]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveResult {
    pub grid: Grid,
    pub gained: u32,
    pub moved: bool,
    /// 本次合并产生的位置(用于弹出动画)
    pub merged: Vec<(usize, usize)>,
    /// 最大合并值(用于飘字)
    pub max_merge: u32,
}

/// 单行向左压缩合并
pub fn slide_row_left(row: [u32; 4]) -> ([u32; 4], u32, Vec<usize>, u32) {
    let mut tiles: Vec<u32> = row.iter().copied().filter(|v| *v != 0).collect();
    let mut gained = 0;
    let mut merged_pos = Vec::new();
    let mut max_merge = 0;
    let mut out = [0u32; 4];
    let mut i = 0;
    let mut o = 0;
    while i < tiles.len() {
        if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
            let merged = tiles[i] * 2;
            gained += merged;
            max_merge = max_merge.max(merged);
            merged_pos.push(o);
            out[o] = merged;
            i += 2;
        } else {
            out[o] = tiles[i];
            i += 1;
        }
        o += 1;
    }
    (out, gained, merged_pos, max_merge)
}

/// 网格向某方向移动
pub fn slide(grid: &Grid, dir: Dir) -> MoveResult {
    let mut new = *grid;
    let mut total_gain = 0u32;
    let mut moved = false;
    let mut merged: Vec<(usize, usize)> = Vec::new();
    let mut max_merge = 0u32;

    for k in 0..4 {
        // 提取一条线(按移动方向从近到远)
        let mut line = [0u32; 4];
        let coords: [(usize, usize); 4] = match dir {
            Dir::Left => [(k, 0), (k, 1), (k, 2), (k, 3)],
            Dir::Right => [(k, 3), (k, 2), (k, 1), (k, 0)],
            Dir::Up => [(0, k), (1, k), (2, k), (3, k)],
            Dir::Down => [(3, k), (2, k), (1, k), (0, k)],
        };
        for (i, (r, c)) in coords.iter().enumerate() {
            line[i] = grid[*r][*c];
        }
        let (out, gained, merged_pos, mm) = slide_row_left(line);
        total_gain += gained;
        max_merge = max_merge.max(mm);
        for (i, (r, c)) in coords.iter().enumerate() {
            if new[*r][*c] != out[i] {
                moved = true;
            }
            new[*r][*c] = out[i];
        }
        for p in merged_pos {
            let (r, c) = coords[p];
            merged.push((r, c));
        }
    }
    MoveResult { grid: new, gained: total_gain, moved, merged, max_merge }
}

pub fn empty_cells(grid: &Grid) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for r in 0..4 {
        for c in 0..4 {
            if grid[r][c] == 0 {
                v.push((r, c));
            }
        }
    }
    v
}

/// 在空格随机生成 2(90%)或 4(10%)
pub fn spawn(grid: &mut Grid) -> Option<(usize, usize)> {
    let cells = empty_cells(grid);
    if cells.is_empty() {
        return None;
    }
    let ix = rand::random::<usize>() % cells.len();
    let (r, c) = cells[ix];
    let val = if rand::random::<f32>() < 0.9 { 2 } else { 4 };
    grid[r][c] = val;
    Some((r, c))
}

pub fn any_move_possible(grid: &Grid) -> bool {
    if !empty_cells(grid).is_empty() {
        return true;
    }
    for dir in Dir::all() {
        let r = slide(grid, dir);
        if r.moved {
            return true;
        }
    }
    false
}

pub fn max_tile(grid: &Grid) -> u32 {
    grid.iter().flat_map(|row| row.iter()).copied().max().unwrap_or(0)
}

/// 序列化(存档/测试)
pub fn to_string(grid: &Grid) -> String {
    grid.iter()
        .map(|row| row.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))
        .collect::<Vec<_>>()
        .join(";")
}

pub fn from_string(s: &str) -> Option<Grid> {
    let rows: Vec<&str> = s.split(';').collect();
    if rows.len() != 4 {
        return None;
    }
    let mut g: Grid = [[0; 4]; 4];
    for (r, row) in rows.iter().enumerate() {
        let cells: Vec<&str> = row.split(',').collect();
        if cells.len() != 4 {
            return None;
        }
        for (c, cell) in cells.iter().enumerate() {
            g[r][c] = cell.parse().ok()?;
        }
    }
    Some(g)
}

/// 初始棋盘:两个随机块
pub fn initial() -> Grid {
    let mut g: Grid = [[0; 4]; 4];
    spawn(&mut g);
    spawn(&mut g);
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(rows: [[u32; 4]; 4]) -> Grid {
        rows
    }

    #[test]
    fn test_slide_row_left_basic() {
        let (out, gained, merged, _) = slide_row_left([2, 2, 4, 0]);
        assert_eq!(out, [4, 4, 0, 0]);
        assert_eq!(gained, 4);
        assert_eq!(merged, vec![0]);
    }

    #[test]
    fn test_slide_row_left_no_double_merge() {
        // 一次移动中每块只合并一次
        let (out, gained, _, _) = slide_row_left([2, 2, 2, 2]);
        assert_eq!(out, [4, 4, 0, 0]);
        assert_eq!(gained, 8);
    }

    #[test]
    fn test_slide_row_left_triple() {
        // 2 2 2 -> 4 2(前两个合并)
        let (out, gained, _, _) = slide_row_left([2, 2, 2, 0]);
        assert_eq!(out, [4, 2, 0, 0]);
        assert_eq!(gained, 4);
    }

    #[test]
    fn test_slide_left() {
        let r = slide(&g([[2, 0, 2, 0], [0, 0, 0, 0], [4, 4, 0, 0], [0, 0, 0, 0]]), Dir::Left);
        assert!(r.moved);
        assert_eq!(r.gained, 12);
        assert_eq!(r.grid[0][0], 4);
        assert_eq!(r.grid[2][0], 8);
    }

    #[test]
    fn test_slide_right() {
        let r = slide(&g([[2, 0, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]), Dir::Right);
        assert!(r.moved);
        assert_eq!(r.grid[0][3], 4);
    }

    #[test]
    fn test_slide_up_down() {
        let r = slide(&g([[0, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0], [0, 2, 0, 0]]), Dir::Up);
        assert!(r.moved);
        assert_eq!(r.grid[0][1], 4);
        assert_eq!(r.grid[1][1], 2);
        let r2 = slide(&g([[0, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0], [0, 2, 0, 0]]), Dir::Down);
        assert_eq!(r2.grid[3][1], 4);
    }

    #[test]
    fn test_no_move_when_same() {
        let grid = g([[2, 4, 8, 16], [32, 64, 128, 256], [2, 4, 8, 16], [32, 64, 128, 256]]);
        for dir in Dir::all() {
            assert!(!slide(&grid, dir).moved, "{dir:?} 应无法移动");
        }
    }

    #[test]
    fn test_game_over_detection() {
        let grid = g([[2, 4, 8, 16], [32, 64, 128, 256], [2, 4, 8, 16], [32, 64, 128, 256]]);
        assert!(!any_move_possible(&grid));
        let grid2 = g([[2, 4, 8, 16], [32, 64, 128, 256], [2, 4, 8, 16], [32, 64, 128, 2]]);
        assert!(!any_move_possible(&grid2));
        let grid3 = g([[2, 4, 8, 16], [32, 64, 128, 256], [2, 4, 8, 16], [32, 64, 128, 256]]);
        assert!(!any_move_possible(&grid3));
    }

    #[test]
    fn test_move_possible_with_merge() {
        let grid = g([[2, 2, 4, 8], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        assert!(any_move_possible(&grid));
    }

    #[test]
    fn test_empty_cells() {
        let mut grid = g([[2, 0, 0, 4], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        assert_eq!(empty_cells(&grid).len(), 14);
        // 填满后为 0
        for r in 0..4 {
            for c in 0..4 {
                if grid[r][c] == 0 {
                    grid[r][c] = 2;
                }
            }
        }
        assert_eq!(empty_cells(&grid).len(), 0);
    }

    #[test]
    fn test_spawn() {
        let mut grid = g([[0; 4]; 4]);
        let p = spawn(&mut grid);
        assert!(p.is_some());
        let (r, c) = p.unwrap();
        assert!(grid[r][c] == 2 || grid[r][c] == 4);
        // 满盘不生成
        let mut full = g([[2; 4]; 4]);
        assert!(spawn(&mut full).is_none());
    }

    #[test]
    fn test_max_tile() {
        let grid = g([[2, 4, 8, 16], [0, 0, 0, 0], [0, 1024, 0, 0], [0, 0, 0, 512]]);
        assert_eq!(max_tile(&grid), 1024);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let grid = g([[2, 0, 4, 16], [32, 0, 0, 256], [0, 8, 0, 0], [2, 4, 8, 2048]]);
        let s = to_string(&grid);
        let back = from_string(&s).unwrap();
        assert_eq!(back, grid);
    }

    #[test]
    fn test_serialize_invalid() {
        assert!(from_string("bad").is_none());
        assert!(from_string("1,2;3").is_none());
        assert!(from_string("1,2,3,4,5;1,2,3,4;1,2,3,4;1,2,3,4").is_none());
    }

    #[test]
    fn test_all_directions_comprehensive() {
        // 8 个方向语义用例:右移 [4,2,2,0] -> [0,0,4,4]
        let r = slide(&g([[4, 2, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]), Dir::Right);
        assert_eq!(r.grid[0], [0, 0, 4, 4]);
        // 上移 [2;2;4;4] 列 -> [4;8;0;0]
        let r = slide(&g([[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]]), Dir::Up);
        assert_eq!(r.grid[0][0], 4);
        assert_eq!(r.grid[1][0], 8);
        // 下移同列 -> [0;0;4;8]
        let r = slide(&g([[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]]), Dir::Down);
        assert_eq!(r.grid[3][0], 8);
        assert_eq!(r.grid[2][0], 4);
    }

    #[test]
    fn test_merge_chain_restriction() {
        // [4,4,8,0] 左移 -> [8,8,0,0](4+4=8 不再与新 8 合并)
        let (out, gained, _, _) = slide_row_left([4, 4, 8, 0]);
        assert_eq!(out, [8, 8, 0, 0]);
        assert_eq!(gained, 8);
    }

    #[test]
    fn test_initial_has_two_tiles() {
        let g0 = initial();
        let non_zero = empty_cells(&g0).len();
        assert_eq!(non_zero, 14);
    }

    #[test]
    fn test_win_condition_value() {
        // 合并到 2048 时 gained 中包含 2048
        let (out, gained, _, _) = slide_row_left([1024, 1024, 0, 0]);
        assert_eq!(out[0], 2048);
        assert_eq!(gained, 2048);
    }
}

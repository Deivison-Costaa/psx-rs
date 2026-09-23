const FRAC: u32 = 12;
const HALF: i64 = 1 << (FRAC - 1);
const MAX_WIDTH: i32 = 1023;
const MAX_HEIGHT: i32 = 511;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Vertex {
    pub x: i32,
    pub y: i32,
    pub color: u32,
    pub u: u8,
    pub v: u8,
}

#[derive(Clone, Copy, Debug)]
struct Plane {
    base: i64,
    step_x: i64,
    step_y: i64,
}

impl Plane {
    fn new(verts: &[Vertex; 3], values: [i32; 3], det: i64) -> Self {
        let [a, b, c] = verts;
        let (d1, d2) = (
            i64::from(values[1] - values[0]),
            i64::from(values[2] - values[0]),
        );
        let (bx, by) = (i64::from(b.x - a.x), i64::from(b.y - a.y));
        let (cx, cy) = (i64::from(c.x - a.x), i64::from(c.y - a.y));
        let num_x = d1 * cy - d2 * by;
        let num_y = d2 * bx - d1 * cx;
        Self {
            base: (i64::from(values[0]) << FRAC) + HALF,
            step_x: (num_x << FRAC) / det,
            step_y: (num_y << FRAC) / det,
        }
    }

    fn at(&self, dx: i64, dy: i64) -> i32 {
        ((self.base + dx * self.step_x + dy * self.step_y) >> FRAC) as i32
    }
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    slope: i64,
    offset: i64,
    ay: i64,
    run: i64,
    threshold: i64,
}

impl Edge {
    fn new(a: &Vertex, b: &Vertex) -> Self {
        let run = i64::from(b.x - a.x);
        let rise = i64::from(b.y - a.y);
        let includes_ties = rise < 0 || (rise == 0 && run > 0);
        Self {
            slope: -rise,
            offset: rise * i64::from(a.x),
            ay: i64::from(a.y),
            run,
            threshold: if includes_ties { 0 } else { 1 },
        }
    }

    fn clamp_row(&self, y: i64, lo: i64, hi: i64) -> (i64, i64) {
        let rest = self.threshold - (self.run * (y - self.ay) + self.offset);
        match self.slope {
            0 if rest > 0 => (0, 0),
            0 => (lo, hi),
            k if k > 0 => (lo.max(div_ceil(rest, k)), hi),
            k => (lo, hi.min((-rest).div_euclid(-k) + 1)),
        }
    }
}

fn div_ceil(n: i64, d: i64) -> i64 {
    -((-n).div_euclid(d))
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Triangle {
    origin_x: i64,
    origin_y: i64,
    edges: [Edge; 3],
    channels: [Plane; 3],
    tex_u: Plane,
    tex_v: Plane,
    pub top: i32,
    pub bottom: i32,
    left: i32,
    right: i32,
}

impl Triangle {
    pub fn setup(verts: [Vertex; 3]) -> Option<Self> {
        let too_big = (0..3).any(|i| {
            let (p, q) = (verts[i], verts[(i + 1) % 3]);
            (p.x - q.x).abs() > MAX_WIDTH || (p.y - q.y).abs() > MAX_HEIGHT
        });
        let [a, b, c] = verts;
        let det = i64::from(b.x - a.x) * i64::from(c.y - a.y)
            - i64::from(c.x - a.x) * i64::from(b.y - a.y);
        if too_big || det == 0 {
            return None;
        }
        let ordered = if det > 0 { [a, b, c] } else { [a, c, b] };
        let edges = [
            Edge::new(&ordered[0], &ordered[1]),
            Edge::new(&ordered[1], &ordered[2]),
            Edge::new(&ordered[2], &ordered[0]),
        ];
        let channel = |shift: u32| {
            let values = verts.map(|v| ((v.color >> shift) & 0xFF) as i32);
            Plane::new(&verts, values, det)
        };
        Some(Self {
            origin_x: i64::from(a.x),
            origin_y: i64::from(a.y),
            edges,
            channels: [channel(0), channel(8), channel(16)],
            tex_u: Plane::new(&verts, verts.map(|v| i32::from(v.u)), det),
            tex_v: Plane::new(&verts, verts.map(|v| i32::from(v.v)), det),
            top: a.y.min(b.y).min(c.y),
            bottom: a.y.max(b.y).max(c.y),
            left: a.x.min(b.x).min(c.x),
            right: a.x.max(b.x).max(c.x) + 1,
        })
    }

    pub fn span(&self, y: i32) -> Option<(i32, i32)> {
        let y = i64::from(y);
        let (lo, hi) = self.edges.iter().fold(
            (i64::from(self.left), i64::from(self.right)),
            |(lo, hi), e| e.clamp_row(y, lo, hi),
        );
        (lo < hi).then_some((lo as i32, hi as i32))
    }

    fn delta(&self, x: i32, y: i32) -> (i64, i64) {
        (i64::from(x) - self.origin_x, i64::from(y) - self.origin_y)
    }

    pub fn color(&self, x: i32, y: i32) -> [i32; 3] {
        let (dx, dy) = self.delta(x, y);
        self.channels.map(|p| p.at(dx, dy).clamp(0, 255))
    }

    pub fn uv(&self, x: i32, y: i32) -> (i32, i32) {
        let (dx, dy) = self.delta(x, y);
        (self.tex_u.at(dx, dy), self.tex_v.at(dx, dy))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vert(x: i32, y: i32, color: u32) -> Vertex {
        Vertex {
            x,
            y,
            color,
            u: 0,
            v: 0,
        }
    }

    #[test]
    fn span_exclui_a_borda_direita_e_a_linha_de_baixo() {
        let tri = Triangle::setup([vert(0, 0, 0), vert(10, 0, 0), vert(0, 10, 0)]);
        let tri = tri.expect("triangulo valido");
        assert_eq!(tri.span(0), Some((0, 10)));
        assert_eq!(tri.span(5), Some((0, 5)));
        assert_eq!(tri.span(10), None);
    }

    #[test]
    fn ordem_dos_vertices_nao_muda_a_cobertura() {
        let a = Triangle::setup([vert(3, 1, 0), vert(17, 9, 0), vert(1, 12, 0)]);
        let b = Triangle::setup([vert(1, 12, 0), vert(17, 9, 0), vert(3, 1, 0)]);
        let (a, b) = (a.expect("a"), b.expect("b"));
        for y in 0..14 {
            assert_eq!(a.span(y), b.span(y), "linha {y}");
        }
    }

    #[test]
    fn degenerado_e_grande_demais_sao_descartados() {
        assert!(Triangle::setup([vert(0, 0, 0), vert(5, 5, 0), vert(10, 10, 0)]).is_none());
        assert!(Triangle::setup([vert(0, 0, 0), vert(1024, 0, 0), vert(0, 5, 0)]).is_none());
        assert!(Triangle::setup([vert(0, 0, 0), vert(5, 0, 0), vert(0, 512, 0)]).is_none());
        assert!(Triangle::setup([vert(0, 0, 0), vert(1023, 0, 0), vert(0, 511, 0)]).is_some());
    }

    #[test]
    fn gradiente_de_u_arredonda_como_o_hardware() {
        let mut verts = [vert(0, 0, 0), vert(129, 0, 0), vert(0, 1, 0)];
        verts[1].u = 1;
        let tri = Triangle::setup(verts).expect("triangulo valido");
        assert_eq!(tri.uv(66, 0).0, 0);
        assert_eq!(tri.uv(67, 0).0, 1);
    }
}

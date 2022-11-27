
use rand::Rng;
use tui::{widgets::canvas::{Shape, Line, Painter}, style::Color};


pub const GRIDSIZE: usize = 100;
pub const ALIVE_COLOR: Color = tui::style::Color::Green;
pub const DEAD_COLOR: Color = tui::style::Color::Black;


#[derive(Debug, Copy, Clone, Default)]
pub struct Cell {
    pub state: bool,
    pub ind: usize,
}

impl Cell {
    pub fn _toggle(&mut self) {
        if self.state {
            self.state = false;
        } else {
            self.state = true;
        }
    }

    pub fn neighbours(&self) -> Vec<usize> {
        let ind = self.ind;
        let mut neighbours: Vec<usize> = Vec::new();
        let mut nes: [usize; 8] = [
            usize::MAX,
            usize::MAX,
            usize::MAX,
            usize::MAX,
            ind + 1,
            ind + 99,
            ind + 100,
            ind + 101,
        ];

        //1,9,10,11
        if ind > 0 {
            nes[0] = ind - 1;
        }
        if ind >= 99 {
            nes[1] = ind - 99;
        }
        if ind >= 100 {
            nes[2] = ind - 100;
        }
        if ind >= 101 {
            nes[3] = ind - 101;
        }
        // println!("{:?}", nes);
        for n in nes {
            if n < (usize::from(GRIDSIZE) * usize::from(GRIDSIZE)) {
                neighbours.push(n)
            }
        }

        neighbours.clone()
    }

    pub fn set_alive(&mut self) {
        self.state = true;
    }
    
    pub fn set_dead(&mut self) {
        self.state = false;
    }


}


#[derive(Clone, Copy)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct cell_rect{
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color,
}

impl Default for cell_rect{
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, width: 1.0, height: 1.0, color: Color::Black }
    }
}


#[allow(non_camel_case_types, non_snake_case)]
pub struct grid {
    pub Cells: [Cell; (GRIDSIZE * GRIDSIZE)],
    pub Nes: Vec<Vec<usize>>,
    pub Dim: [cell_rect; (GRIDSIZE * GRIDSIZE)]
}


impl grid {
    pub fn new() -> grid {
        let mut grid = grid {
            Cells: [Cell::default(); 10000],
            Nes: Vec::new(),
            Dim: [cell_rect::default(); 10000]
        };
        let mut ind = 0;
        let mut xd:f64 = 0.0;
       
        
        for _r in 0..100 {
            let mut yd:f64 = 0.0;
            for _c in 0..100 {
                let cell = Cell {
                    state: false,
                    ind: ind
                };

                let rect = cell_rect{
                    x: xd,
                    y: yd, 
                    width: 1.0, 
                    height: 1.0, 
                    color: Color::Black};

                yd += 3.0;
                grid.Cells[ind] = cell;
                grid.Nes.push( grid.Cells[ind].neighbours());
                grid.Dim[ind] = rect.clone();
                ind += 1;
            }
            xd += 3.0;
        }
        grid
    }
    
    pub fn alone(&self, alive_nes_count: usize) -> bool {
        alive_nes_count < 2
    }

    pub fn overpopulated(&self, alive_nes_count: usize) -> bool {
        alive_nes_count > 3
    }

    pub fn can_be_revived(&self, alive_nes_count: usize) -> bool {
        alive_nes_count == 3
    }

    pub fn random_gen(&mut self){
        // info!("Random Gen called Called for GRID");
        let mut rng = rand::thread_rng();
        let mut ind = 0;
        for _i in 0..10000{
            if rng.gen() {
                self.Cells[ind].set_alive();
            } else {
                self.Cells[ind].set_dead();
            }
            ind += 1;
        }        
        
    }

    pub fn _display(&mut self) {
        let mut ind: usize = 0;
        for _r in 0..GRIDSIZE {
            for _c in 0..GRIDSIZE {
                println!("{:?}", self.Cells[ind]);
                ind += 1;
            }
        }

        let indexes = [0, 99, 9990, 9999, 550];

        for ind in indexes {
            let nes = self.Cells[ind].neighbours();
            println!(
                "All: {:?}",
                nes.clone(),
            );
        }
    }

    pub fn _show(&self) {
        let mut ind: usize = 0;
        for _i in 0..100{
            for _j in 0..100{
                if self.Cells[ind].state {
                    print!("1");
                } else {
                    print!("0");
                }
                ind += 1;    
            }
            println!("");
        }
        println!("Printed The Cells");
        println!();


    }
    
    pub fn _show_dim(&self){
        for d in self.Dim{
            println!("x: {}, y:{}", d.x, d.y);    
        }
    }
    // fn next_gen(&mut self) -> Vec<(usize, bool)> {}
    pub fn on_tick(&mut self, active: bool) {
        // info!("On Tick Called for GRID");
       
        if active {
            //caclulate next gen
            // info!("Cell States : {:?} ", self.Cells);
            let mut to_dead: Vec<usize> = Vec::new();
            let mut to_live: Vec<usize> = Vec::new();
            for i in 0..10000{
                let mut alive_nes: Vec<usize> = Vec::new();
                let mut dead_nes: Vec<usize> = Vec::new();
                let nes = self.Nes[i].clone();
                for n in nes{
                    if self.Cells[n].state{
                        alive_nes.push(n.clone());
                    } else{
                        dead_nes.push(n.clone());
                    }
                }

                if self.Cells[i].state {
                    if self.alone(alive_nes.len()) || self.overpopulated(alive_nes.len()) {
                        to_dead.push(i);
                    }
                } else if self.can_be_revived(alive_nes.len()) {
                    to_live.push(i);
                }
                
                // info!("Alive Cells : {} ", to_live.len());
                // info!("Dead Cells : {} ", to_dead.len());
        

                for ind in &to_live{
                    self.Cells[*ind].set_alive();
                }
                for ind in &to_dead{
                    self.Cells[*ind].set_dead();
                }


            }
        } else{
            // info!("Active False Skipping next gen calculation")
        }

    }

    fn _draw_rects(&self, painter: &mut Painter){
        let mut ind = 0;
        for mut dim in self.Dim{
            if self.Cells[ind].state{
                dim.color = ALIVE_COLOR;
            let lines: [Line; 4] = [
                Line {
                    x1: dim.x,
                    y1: dim.y,
                    x2: dim.x,
                    y2: dim.y + dim.height,
                    color: dim.color,
                },
                Line {
                    x1: dim.x,
                    y1: dim.y + dim.height,
                    x2: dim.x + dim.width,
                    y2: dim.y + dim.height,
                    color: dim.color,
                },
                Line {
                    x1: dim.x + dim.width,
                    y1: dim.y,
                    x2: dim.x + dim.width,
                    y2: dim.y + dim.height,
                    color: dim.color,
                },
                Line {
                    x1: dim.x,
                    y1: dim.y,
                    x2: dim.x + dim.width,
                    y2: dim.y,
                    color: dim.color,
                },
               ];
            for line in &lines {
                    line.draw(painter);
                }
            }
            ind += 1;
        }
    }
    
    fn _draw_points(&self, painter: &mut Painter){
        let mut ind = 0;
        for x in 0..100{
            for y in 0..100{
                if self.Cells[ind].state{
                    painter.paint(x, y, ALIVE_COLOR);
                } else{
                    painter.paint(x, y, DEAD_COLOR);
                }
                ind += 1;
            }
        }
    }

}



impl Shape for grid{
    fn draw(&self, painter: &mut Painter) {
        self._draw_points(painter);
    }
}


/*
[0](0,0)  [1](0,1)  [2](0,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [9](9,9)
[10](1,0) [11](1,1) [12](1,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [19](9,9)
[20](2,0) [21](2,1) [22](2,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [29](9,9)
[30](3,0) [31](3,1) [32](3,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [39](9,9)
[40](4,0) [41](4,1) [42](4,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [49](9,9)
[50](5,0) [51](5,1) [52](5,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [59](9,9)
[60](6,0) [61](6,1) [62](6,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [69](9,9)
[70](7,0) [71](7,1) [72](7,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](0,8)   [79](9,9)
[80](8,0) [81](8,1) [82](8,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](8,8)   [89](9,9)
[90](9,0) [91](9,1) [92](9,2) [3](0,3)  [4](0,4)  [5](0,5)  [6](0,6)  [7](0,7)   [8](8,8)   [99](9,9)




*/
#[cfg(test)]
mod test{
    use super::grid;

    #[test]
    fn test_rand(){
        let mut g = grid::new();
        g._show();
        g.random_gen();
        g._show();
        g.on_tick(true);
        g._show();

    }
    #[test]
    fn test_rect(){
        let mut g = grid::new();
        g.random_gen();
        g._show_dim();

    }
}

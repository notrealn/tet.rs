use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, Result,
};
use fastrand::shuffle;
use std::io;
use std::time::Duration;
use std::vec::Vec;

fn main() -> Result<()> {
    enable_raw_mode()?;

    loop {
        match title_screen()? {
            TitleSelectOptions::Start => game_screen()?,
            TitleSelectOptions::Controls => controls_screen()?,
            TitleSelectOptions::Exit => break,
        }
    }

    disable_raw_mode()?;
    return Ok(());
}

fn title_screen() -> Result<TitleSelectOptions> {
    let title = "\r ______       __  \r\n/_  __/ ___  / /_     ____  ___\r\n / /   / -_)/ __/ _  / __/ (_-<\r\n/_/    \\__/ \\__/ (_)/_/   /___/";
    let mut selected = TitleSelectOptions::Start;

    loop {
        io::stdout()
            .execute(Clear(ClearType::All))?
            .execute(Print(title))?
            .execute(Print("\r\nTetris in the command line, made in rust.\r\nUse A/D/ENTER to navigate menus.\r\n\n"))?
            .execute(Print(match selected {
                TitleSelectOptions::Start => "[Start] Controls Exit",
                TitleSelectOptions::Controls => "Start [Controls] Exit",
                TitleSelectOptions::Exit => "Start Controls [Exit]",
            }))?;

        match read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Char('c')
                    && event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                };

                use TitleSelectOptions::*;
                match event.code {
                    KeyCode::Char('a') => {
                        selected = match selected {
                            Start => Exit,
                            Controls => Start,
                            Exit => Controls,
                        }
                    }
                    KeyCode::Char('d') => {
                        selected = match selected {
                            Start => Controls,
                            Controls => Exit,
                            Exit => Start,
                        }
                    }
                    KeyCode::Enter => return Ok(selected),
                    _ => (),
                }
            }
            Event::Mouse(_) => (),
            Event::Resize(_, _) => (),
        }
    }

    Ok(TitleSelectOptions::Exit)
}

fn controls_screen() -> Result<()> {
    io::stdout()
        .execute(Clear(ClearType::All))?
        .execute(Print("\r  _____          __           __  \r\n / ___/__  ___  / /________  / /__\r\n/ /__/ _ \\/ _ \\/ __/ __/ _ \\/ (_-<\r\n\\___/\\___/_//_/\\__/_/  \\___/_/___/\r\n"))?
        .execute(Print("\r(press any key to leave)\r\n"))?
        .execute(Print(
            [
                "a:\tsoft move left",
                "d:\tsoft move right",
                "q:\thard left",
                "e:\thard right",
                "s:\tsoft drop",
                "space:\thard drop",
                "j:\trotate left",
                "k:\trotate right",
                "l:\trotate twice",
                ";:\thold piece",
                "\r\nYou can also use ctr+c to exit or end the game.",
            ]
            .join("\r\n"),
        ))?;

    loop {
        match read()? {
            Event::Key(_) => return Ok(()),
            Event::Mouse(_) => (),
            Event::Resize(_, _) => (),
        }
    }
}

fn game_screen() -> Result<()> {
    io::stdout()
        .execute(EnterAlternateScreen)?
        .execute(Clear(ClearType::All))?;

    // use Cell::*;
    // [x][y]
    let mut state = GameState {
        matrix: [[Tile::None; 20]; 10],
        tetrimino: generate_tetrimino(Tile::None),
        game_over: false,
        can_hold: true,
        held: Tile::None,
        bag1: generate_bag(),
        bag2: generate_bag(),
        index: 0,
        lines_cleared: 0,
    };

    next_piece(&mut state);

    let mut ticks_till_next_gravity = 60;

    loop {
        io::stdout()
            .execute(MoveTo(0, 0))?
            .execute(Clear(ClearType::All))?
            .execute(Print(state_to_string(&state)))?;

        if state.game_over {
            break;
        }

        if matches!(state.tetrimino.name, Tile::None) {
            next_piece(&mut state);
            state.can_hold = false;
        }

        for row in 0..20 {
            let mut all_filled = true;
            for col in 0..10 {
                if matches!(state.matrix[col][row], Tile::None) {
                    all_filled = false;
                }
            }
            if all_filled {
                clear_row(&mut state, row as i8)
            }
        }

        if ticks_till_next_gravity <= 0 {
            ticks_till_next_gravity = 60;
            if !move_tetrimino(&mut state, Point(0, 1)) {
                solidify_tetrimino(&mut state)
            }
        } else {
            ticks_till_next_gravity -= 1;
        }

        // let mut key;
        if poll(Duration::from_nanos(16666666))? {
            match read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('c') => {
                        if event.modifiers.contains(KeyModifiers::CONTROL) {
                            state.game_over = true;
                        }
                    }
                    KeyCode::Char('a') => {
                        move_tetrimino(&mut state, Point(-1, 0));
                    }
                    KeyCode::Char('d') => {
                        move_tetrimino(&mut state, Point(1, 0));
                    }
                    KeyCode::Char('s') => {
                        move_tetrimino(&mut state, Point(0, 1));
                    }
                    KeyCode::Char('q') => while move_tetrimino(&mut state, Point(-1, 0)) {},
                    KeyCode::Char('e') => while move_tetrimino(&mut state, Point(1, 0)) {},
                    KeyCode::Char(' ') => {
                        while move_tetrimino(&mut state, Point(0, 1)) {}
                        solidify_tetrimino(&mut state);
                        ticks_till_next_gravity = 60
                    }
                    KeyCode::Char('j') => {
                        transform_tetrimino(&mut state, Point(0, 0), Direction::Left);
                    }
                    KeyCode::Char('k') => {
                        transform_tetrimino(&mut state, Point(0, 0), Direction::Right);
                    }
                    KeyCode::Char('l') => {
                        transform_tetrimino(&mut state, Point(0, 0), Direction::Double);
                    }
                    KeyCode::Char(';') => {
                        if state.can_hold {
                            state.can_hold = false;
                            let foo = state.held.clone();
                            state.held = state.tetrimino.name.clone();
                            let possible_tetrimino = generate_tetrimino(foo);
                            if check_tetrimino_overlap(&possible_tetrimino, &state) {
                                state.game_over = true
                            }
                            state.tetrimino = possible_tetrimino;
                        }
                    }
                    _ => (),
                },
                Event::Mouse(_event) => (),
                Event::Resize(_width, _height) => (),
            }
        }
    }

    loop {
        match read()? {
            Event::Key(event) => match event.code {
                KeyCode::Enter => break,
                _ => (),
            },
            Event::Mouse(_) => (),
            Event::Resize(_, _) => (),
        }
    }
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn state_to_string(state: &GameState) -> String {
    // each string is a row of output
    let mut out: [String; 20] = Default::default();

    for row in 0..20 {
        out[row].push('|');
        for col in 0..10 {
            if tetrimino_at_coords(&state.tetrimino, Point(col, row as i8)) {
                out[row].push(tile_to_char(&state.tetrimino.name));
            } else {
                let cell = state.matrix[col as usize][row];
                out[row].push(tile_to_char(&cell));
            }
            out[row].push('|');
        }
    }

    out[0].push_str(&format!(" Next: {}", get_next_5(&state)));
    out[2].push_str(&format!(
        " Held: {}, Can hold: {}",
        match state.held {
            Tile::None => String::from("None"),
            _ => tile_to_char(&state.held).to_string(),
        },
        state.can_hold
    ));
    out[4].push_str(&format!(" Lines cleared: {}", state.lines_cleared));

    if state.game_over {
        out[6].push_str(" Game over! Press enter to continue.");
    }

    out.join("\r\n")
}

fn tile_to_char(tetrimino: &Tile) -> char {
    use Tile::*;
    match tetrimino {
        I => 'I',
        O => 'O',
        T => 'T',
        S => 'S',
        Z => 'Z',
        J => 'J',
        L => 'L',
        None => ' ',
    }
}

fn generate_bag() -> [Tile; 7] {
    use Tile::*;
    let mut bag = [I, O, T, S, Z, J, L];
    shuffle(&mut bag);
    bag
}

fn generate_tetrimino(name: Tile) -> Tetrimino {
    use Tile::*;
    match name {
        I => Tetrimino {
            name,
            shape: vec![
                vec![false, true, false, false],
                vec![false, true, false, false],
                vec![false, true, false, false],
                vec![false, true, false, false],
            ],
            center: Point(1, 1),
            position: Point(4, 0),
        },
        O => Tetrimino {
            name,
            shape: vec![vec![true, true], vec![true, true]],
            center: Point(0, 0),
            position: Point(4, 0),
        },
        T => Tetrimino {
            name,
            shape: vec![
                vec![false, true, false],
                vec![true, true, false],
                vec![false, true, false],
            ],
            center: Point(1, 1),
            position: Point(4, 1),
        },
        S => Tetrimino {
            name,
            shape: vec![
                vec![false, true, false],
                vec![true, true, false],
                vec![true, false, false],
            ],
            center: Point(1, 1),
            position: Point(4, 1),
        },
        Z => Tetrimino {
            name,
            shape: vec![
                vec![true, false, false],
                vec![true, true, false],
                vec![false, true, false],
            ],
            center: Point(1, 1),
            position: Point(4, 1),
        },
        J => Tetrimino {
            name,
            shape: vec![
                vec![true, true, false],
                vec![false, true, false],
                vec![false, true, false],
            ],
            center: Point(1, 1),
            position: Point(4, 1),
        },
        L => Tetrimino {
            name,
            shape: vec![
                vec![false, true, false],
                vec![false, true, false],
                vec![true, true, false],
            ],
            center: Point(1, 1),
            position: Point(4, 1),
        },
        None => Tetrimino {
            name,
            shape: vec![vec![]],
            center: Point(0, 0),
            position: Point(0, 0),
        },
    }
}

// returns false if it couldnt be moved.
fn move_tetrimino(state: &mut GameState, offset: Point) -> bool {
    let possible_tetrimino = generate_offset_tetrimino(&state.tetrimino, offset);
    if !check_tetrimino_overlap(&possible_tetrimino, &state) {
        state.tetrimino = possible_tetrimino;
        return true;
    }
    false
}

fn transform_tetrimino(state: &mut GameState, offset: Point, rotation: Direction) -> bool {
    let possible_tetrimino = generate_transformed_tetrimino(&state.tetrimino, offset, rotation);
    if !check_tetrimino_overlap(&possible_tetrimino, &state) {
        state.tetrimino = possible_tetrimino;
        return true;
    }
    false
}

fn tetrimino_at_coords(tetrimino: &Tetrimino, coords: Point) -> bool {
    let x = coords.0 - tetrimino.position.0 + tetrimino.center.0;
    // check if x is in bounds
    if x >= 0 && x < tetrimino.shape.len() as i8 {
        let y = coords.1 - tetrimino.position.1 + tetrimino.center.1;
        // check if y is in bounds
        if y >= 0 && y < tetrimino.shape[x as usize].len() as i8 {
            return tetrimino.shape[x as usize][y as usize];
        }
    }
    false
}

fn check_tetrimino_overlap(tetrimino: &Tetrimino, state: &GameState) -> bool {
    for Point(x, y) in generate_tetrimino_iterable(&tetrimino) {
        if x >= state.matrix.len() as i8
            || x < 0
            || y >= state.matrix[x as usize].len() as i8
            || y < 0
        {
            return true;
        } else if !matches!(state.matrix[x as usize][y as usize], Tile::None) {
            return true;
        }
    }
    false
}

fn generate_offset_tetrimino(tetrimino: &Tetrimino, offset: Point) -> Tetrimino {
    Tetrimino {
        name: tetrimino.name.clone(),
        shape: tetrimino.shape.clone(),
        center: tetrimino.center.clone(),
        position: Point(
            tetrimino.position.0 + offset.0,
            tetrimino.position.1 + offset.1,
        ),
    }
}

fn generate_transformed_tetrimino(
    tetrimino: &Tetrimino,
    transform: Point,
    rotation: Direction,
) -> Tetrimino {
    let mut shape = transpose_matrix(&tetrimino.shape);
    let position = Point(
        tetrimino.position.0 + transform.0,
        tetrimino.position.1 + transform.1,
    );

    match rotation {
        Direction::Left => {
            for row in 0..shape.len() {
                shape[row].reverse();
            }
        }
        Direction::Right => {
            shape.reverse();
        }
        Direction::Double => {
            shape = transpose_matrix(&shape);
            for row in 0..shape.len() {
                shape[row].reverse();
            }
            shape.reverse();
        }
    }

    Tetrimino {
        name: tetrimino.name.clone(),
        shape,
        center: tetrimino.center.clone(),
        position,
    }
}

fn solidify_tetrimino(state: &mut GameState) {
    for Point(x, y) in generate_tetrimino_iterable(&state.tetrimino) {
        state.matrix[x as usize][y as usize] = state.tetrimino.name
    }
    next_piece(state)
}

fn next_piece(state: &mut GameState) {
    state.can_hold = true;
    if state.index > 6 {
        state.index = 0;
        state.bag1 = state.bag2;
        state.bag2 = generate_bag();
    }
    let possible_tetrimino = generate_tetrimino(state.bag1[state.index as usize]);
    if check_tetrimino_overlap(&possible_tetrimino, &state) {
        state.game_over = true
    }
    state.tetrimino = possible_tetrimino;
    state.index += 1;
}

fn get_next_5(state: &GameState) -> String {
    let mut string = String::new();
    for i in state.index..(state.index + 5) {
        string.push(tile_to_char(&if i < (state.bag1.len() as u8) {
            state.bag1[i as usize]
        } else {
            state.bag2[i as usize - state.bag1.len()]
        }));
        string.push(' ');
    }
    string
}

fn clear_row(state: &mut GameState, row: i8) {
    state.lines_cleared += 1;
    for x in 0..state.matrix.len() {
        for y in (0..(row)).rev() {
            state.matrix[x][y as usize + 1] = state.matrix[x][y as usize]
        }
        state.matrix[x][0] = Tile::None
    }
}

fn generate_tetrimino_iterable(tetrimino: &Tetrimino) -> Vec<Point> {
    let mut vec = Vec::new();

    for x in 0..tetrimino.shape.len() {
        for y in 0..tetrimino.shape[x].len() {
            if tetrimino.shape[x][y] {
                vec.push(Point(
                    x as i8 + tetrimino.position.0 - tetrimino.center.0,
                    y as i8 + tetrimino.position.1 - tetrimino.center.1,
                ))
            }
        }
    }

    vec
}

fn transpose_matrix(m: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut t = vec![vec![]];
    for col in 0..m.len() {
        for row in 0..m[col].len() {
            if t.len() <= row {
                t.push(vec![])
            }
            t[row].push(m[col][row])
        }
    }
    t
}

struct GameState {
    matrix: [[Tile; 20]; 10],
    tetrimino: Tetrimino,
    game_over: bool,
    can_hold: bool,
    held: Tile,
    bag1: [Tile; 7],
    bag2: [Tile; 7],
    index: u8,
    lines_cleared: i32,
}

// x, y (0, 0 is top left)
// offset is placed at 5th column
struct Tetrimino {
    name: Tile,
    shape: Vec<Vec<bool>>,
    center: Point,
    position: Point,
}

#[derive(Copy, Clone)]
struct Point(i8, i8);

#[derive(Copy, Clone)]
enum Tile {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
    None,
}

enum Direction {
    Left,
    Right,
    Double,
}

enum TitleSelectOptions {
    Start,
    Controls,
    Exit,
}

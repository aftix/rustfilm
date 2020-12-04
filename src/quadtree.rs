pub struct QuadTree<T: Clone> {
  data: Option<T>,
  q1: Option<Box<QuadTree<T>>>,
  q2: Option<Box<QuadTree<T>>>,
  q3: Option<Box<QuadTree<T>>>,
  q4: Option<Box<QuadTree<T>>>,
  leaf: bool,
  xmin: f64,
  xmax: f64,
  ymin: f64,
  ymax: f64,
  x: f64,
  y: f64,
}

impl<T: Clone> QuadTree<T> {
  pub fn new(xmin: f64, xmax: f64, ymin: f64, ymax: f64) -> QuadTree<T> {
    if xmin >= xmax {
      panic!("xmin must be less than xmax");
    }

    if ymin >= ymax {
      panic!("ymin must be less than ymax");
    }

    QuadTree{
      data: None,
      q1: None,
      q2: None,
      q3: None,
      q4: None,
      leaf: true,
      xmin,
      xmax,
      ymin,
      ymax,
      x: 0.0,
      y: 0.0,
    }
  }

  pub fn add(&mut self, item: T, x: f64, y: f64) {
    if x < self.xmin {
      panic!("x too small");
    } else if x > self.xmax {
      panic!("x too big");
    } else if y < self.ymin {
      panic!("y too small");
    } else if y > self.ymax {
      panic!("y too big");
    }

    if self.leaf { // Is leaf, needs logic
      match &self.data { // Empty, fill it and set position
        None => {
          self.x = x;
          self.y = y;
          self.data = Some(item);
        },
        Some(data) => { // Full, Split and cascade
          let xmid = 0.5 * (self.xmax + self.xmin);
          let ymid = 0.5 * (self.ymax + self.ymin);

          // Make four children
          self.q1 = Some(Box::new(QuadTree::new(
            xmid,
            self.xmax,
            ymid,
            self.ymax,
          )));

          self.q2 = Some(Box::new(QuadTree::new(
            self.xmin,
            xmid,
            ymid,
            self.ymax,
          )));

          self.q3 = Some(Box::new(QuadTree::new(
            self.xmin,
            xmid,
            self.ymin,
            ymid,
          )));

          self.q4 = Some(Box::new(QuadTree::new(
            xmid,
            self.xmax,
            self.ymin,
            ymid,
          )));

          // Put current data to children
          if x > xmid && y > ymid {
            if let Some(q1) = &mut self.q1 {
              q1.data = Some(data.clone());
            }
          } else if x < xmid && y > ymid {
            if let Some(q2) = &mut self.q2 {
              q2.data = Some(data.clone());
            }
          } else if x < xmid && y < ymid {
            if let Some(q3) = &mut self.q3 {
              q3.data = Some(data.clone());
            }
          } else if x > xmid && y < ymid {
            if let Some(q4) = &mut self.q4 {
              q4.data = Some(data.clone());
            }
          }

          if x > xmid && y > ymid {
            if let Some(q1) = &mut self.q1 {
              q1.add(item, x, y);
            }
          } else if x < xmid && y > ymid {
            if let Some(q2) = &mut self.q2 {
              q2.add(item, x, y);
            }
          } else if x < xmid && y < ymid {
            if let Some(q3) = &mut self.q3 {
              q3.add(item, x, y);
            }
          } else if x > xmid && y < ymid {
            if let Some(q4) = &mut self.q4 {
              q4.add(item, x, y);
            }
          }
        }
      }
    } else { // Not a leaf node, pass to children
      let xmid = 0.5 * (self.xmax + self.xmin);
      let ymid = 0.5 * (self.ymax + self.ymin);

      if x > xmid && y > ymid {
        if let Some(q1) = &mut self.q1 {
          q1.add(item, x, y);
        }
      } else if x < xmid && y > ymid {
        if let Some(q2) = &mut self.q2 {
          q2.add(item, x, y);
        }
      } else if x < xmid && y < ymid {
        if let Some(q3) = &mut self.q3 {
          q3.add(item, x, y);
        }
      } else if x > xmid && y < ymid {
        if let Some(q4) = &mut self.q4 {
          q4.add(item, x, y);
        }
      }
    }
  }

  pub fn get_within(&self, x: f64, y: f64, radius: f64) -> Vec<T> {
    let mut within = vec![];

    if self.leaf {
      if let Some(data) = &self.data {
        if ((self.x - x).powi(2) + (self.y - y).powi(2)).sqrt() <= radius {
          within.push(data.clone());
        }
      }
      return within;
    }

    let xmid = 0.5 * (self.xmax + self.xmin);
    let ymid = 0.5 * (self.ymax + self.ymin);

    if test_circle_rect((x, y, radius), (xmid, self.xmax, ymid, self.ymax)) {
      if let Some(q1) = &self.q1 {
        let got = q1.get_within(x, y, radius);
        for item in got {
          within.push(item);
        }
      }
    }

    if test_circle_rect((x, y, radius), (self.xmin, xmid, ymid, self.ymax)) {
      if let Some(q2) = &self.q2 {
        let got = q2.get_within(x, y, radius);
        for item in got {
          within.push(item);
        }
      }
    }

    if test_circle_rect((x, y, radius), (self.xmin, xmid, self.ymin, ymid)) {
      if let Some(q3) = &self.q3 {
        let got = q3.get_within(x, y, radius);
        for item in got {
          within.push(item);
        }
      }
    }

    if test_circle_rect((x, y, radius), (xmid, self.xmax, self.ymin, ymid)) {
      if let Some(q4) = &self.q4 {
        let got = q4.get_within(x, y, radius);
        for item in got {
          within.push(item);
        }
      }
    }

    within
  }
}

fn test_circle_rect((circle_x, circle_y, circle_rad): (f64, f64, f64), (rect_left, rect_right, rect_top, rect_bottom): (f64, f64, f64, f64)) -> bool {
  let test_x = if circle_x < rect_left { rect_left } else if circle_x > rect_right { rect_right } else { circle_x };
  let test_y = if circle_y < rect_bottom { rect_bottom } else if circle_y > rect_top { rect_top } else { circle_y };

  let dist_x = circle_x - test_x;
  let dist_y = circle_y - test_y;
  let dist = (dist_x*dist_x + dist_y*dist_y).sqrt();

  dist <= circle_rad
}

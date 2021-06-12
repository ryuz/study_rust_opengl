use num_traits::{Float, FromPrimitive, ToPrimitive};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

// 即値生成
fn get<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

// 文字列パース
fn parse_number<T: FromStr>(stri: &str, line_num: i32) -> Result<T, String> {
    match stri.parse() {
        Ok(result) => Ok(result),
        Err(_) => Err(format!("line[{}]: parse error", line_num).to_string()),
    }
}

// 3D座標
#[allow(dead_code)]
pub struct Vecter3D<T: FromPrimitive> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[allow(dead_code)]
impl<T: FromPrimitive> Vecter3D<T> {
    fn new() -> Self {
        Vecter3D::<T> {
            x: get::<T>(0.0),
            y: get::<T>(0.0),
            z: get::<T>(0.0),
        }
    }
}

// テクスチャ座標
#[allow(dead_code)]
pub struct Texture2D<T: FromPrimitive> {
    pub u: T,
    pub v: T,
}

#[allow(dead_code)]
impl<T: FromPrimitive> Texture2D<T> {
    fn new() -> Self {
        Texture2D::<T> {
            u: get::<T>(0.0),
            v: get::<T>(0.0),
        }
    }
}

// マテリアル情報
#[allow(dead_code)]
pub struct Material<T: FromPrimitive> {
    pub material_name: String,
    pub diffuse: Vecter3D<T>,
    pub ambient: Vecter3D<T>,
    pub specular: Vecter3D<T>,
    pub shininess: T,
    pub alpha: T,
    pub diffuse_filename: String,
    pub ambient_filename: String,
    pub specular_filename: String,
    pub bumpmap_filename: String,
}

#[allow(dead_code)]
impl<T: FromPrimitive> Material<T> {
    fn new() -> Self {
        Material::<T> {
            material_name: String::new(),
            diffuse: Vecter3D::<T>::new(),
            ambient: Vecter3D::<T>::new(),
            specular: Vecter3D::<T>::new(),
            shininess: get::<T>(0.0),
            alpha: get::<T>(0.0),
            diffuse_filename: String::new(),
            ambient_filename: String::new(),
            specular_filename: String::new(),
            bumpmap_filename: String::new(),
        }
    }

    fn empty(&self) -> bool {
        self.material_name.len() == 0
    }
}

// 点
#[allow(dead_code)]
struct Point {
    vertex_index: i32,
    normal_index: i32,
    texture_coordinate_index: i32,
}

#[allow(dead_code)]
impl Point {
    fn new() -> Self {
        Point {
            vertex_index: 0,
            normal_index: 0,
            texture_coordinate_index: 0,
        }
    }
}

// 面
#[allow(dead_code)]
struct Face {
    points: Vec<Point>,
}

#[allow(dead_code)]
impl Face {
    fn new() -> Self {
        Face { points: Vec::new() }
    }

    fn empty(&self) -> bool {
        self.points.len() == 0
    }
}

// サーフェース(同一マテリアル単位のFace群)
#[allow(dead_code)]
struct Surface {
    material_index: i32,
    faces: Vec<Face>,
}

#[allow(dead_code)]
impl Surface {
    fn new() -> Self {
        Surface {
            material_index: -1,
            faces: Vec::new(),
        }
    }

    fn empty(&self) -> bool {
        self.faces.len() == 0
    }
}

// グループ
#[allow(dead_code)]
struct Group {
    group_name: String,     // グループ名
    surfaces: Vec<Surface>, // ポリゴン面
}

#[allow(dead_code)]
impl Group {
    fn new() -> Self {
        Group {
            group_name: String::new(),
            surfaces: Vec::new(),
        }
    }

    fn empty(&self) -> bool {
        self.surfaces.len() == 0
    }
}

// オブジェクト
#[allow(dead_code)]
struct Object {
    object_name: String, // オブジェクト名
    groups: Vec<Group>,  // ポリゴングループ
}

#[allow(dead_code)]
impl Object {
    fn new() -> Self {
        Object {
            object_name: String::new(),
            groups: Vec::new(),
        }
    }

    fn empty(&self) -> bool {
        self.groups.len() == 0
    }
}

// メッシュ
#[allow(dead_code)]
pub struct Mesh<T: FromPrimitive> {
    mesh_name: String,
    objects: Vec<Object>,

    vertexes: Vec<Vecter3D<T>>,             // 頂点座標リスト
    texture_coordinates: Vec<Texture2D<T>>, // テクスチャ座標リスト
    normals: Vec<Vecter3D<T>>,              // 法線ベクトルリスト
    materials: Vec<Material<T>>,            // マテリアル
}

#[allow(dead_code)]
impl<T: FromPrimitive> Mesh<T> {
    fn new() -> Self {
        Mesh::<T> {
            mesh_name: String::new(),
            objects: Vec::new(),

            vertexes: Vec::new(),
            texture_coordinates: Vec::new(),
            normals: Vec::new(),
            materials: Vec::new(),
        }
    }

    fn empty(&self) -> bool {
        self.objects.len() == 0
    }
}

#[allow(dead_code)]
impl<T: FromStr + Float + FromPrimitive + ToPrimitive> Mesh<T> {
    pub fn load(filename: &str) -> Result<Box<Mesh<T>>, String> {
        // ファイルオープン
        let f = match File::open(&filename) {
            Err(_) => {
                return Err(format!("couldn't open {}", filename));
            }
            Ok(file) => file,
        };

        // オブジェクト準備
        let mut mesh = Box::new(Mesh::<T>::new());
        let mut obj = Object::new();
        let mut grp = Group::new();
        let mut surf = Surface::new();

        let mut line_num: i32 = 0;

        // 行単位で処理
        let reader = BufReader::new(f);
        for line in reader.lines() {
            line_num += 1;

            // 空白で分解
            let text: &str = &line.unwrap();
            let params: Vec<&str> = text.trim().split(" ").collect();

            // 空行 or コメント行ならスキップ
            if params.len() <= 0 || params[0].len() <= 0 || params[0].chars().nth(0).unwrap() == '#'
            {
                continue;
            }

            // コマンドと引数に分解
            let command: &str = params[0];
            let args: &[&str] = &params[1..];

            match (command, args.len()) {
                ("o", 1) => {
                    // オブジェクト
                    if !obj.empty() {
                        mesh.objects.push(obj);
                        obj = Object::new();
                    }
                    obj.object_name = args[0].to_string();
                }

                ("g", 1) => {
                    // グループ
                    if !surf.empty() {
                        grp.surfaces.push(surf);
                        surf = Surface::new();
                    }
                    if !grp.empty() {
                        obj.groups.push(grp);
                    }

                    grp = Group::new();
                    grp.group_name = args[0].to_string();
                }

                ("mtllib", 1) => {
                    // マテリアルファイル読み込み＆登録
                    Mesh::load_mtl(&mut mesh, args[0])?;
                }

                ("usemtl", 1) => {
                    // サーフェース登録
                    if !surf.empty() {
                        grp.surfaces.push(surf);
                    }
                    surf = Surface::new();

                    // 利用マテリアル検索
                    for (i, mat) in mesh.materials.iter().enumerate() {
                        if mat.material_name == args[0] {
                            surf.material_index = i as i32;
                            break;
                        }
                    }
                }

                ("v", 3) => {
                    // 頂点情報
                    mesh.vertexes.push(Vecter3D::<T> {
                        x: parse_number(args[0], line_num)?,
                        y: parse_number(args[1], line_num)?,
                        z: parse_number(args[2], line_num)?,
                    })
                }

                ("vt", 2) => {
                    // テクスチャ座標
                    mesh.texture_coordinates.push(Texture2D::<T> {
                        u: parse_number(args[0], line_num)?,
                        v: get::<T>(1.0) - parse_number(args[1], line_num)?, // 左下原点(OpenGL座標)に変換
                    })
                }

                ("vn", 3) => {
                    // 法線情報
                    mesh.normals.push(Vecter3D::<T> {
                        x: parse_number(args[0], line_num)?,
                        y: parse_number(args[1], line_num)?,
                        z: parse_number(args[2], line_num)?,
                    })
                }

                ("f", n) => {
                    // 面
                    if n < 3 {
                        return Err(format!("[{}: There are too few points", line_num).to_string());
                    }

                    let mut face = Face::new();
                    for arg in args {
                        let indexes: Vec<&str> = arg.split("/").collect();
                        face.points.push(Point {
                            vertex_index: if indexes.len() > 0 {
                                parse_number::<i32>(indexes[0], line_num)? - 1
                            } else {
                                -1
                            },
                            texture_coordinate_index: if indexes.len() > 1 {
                                parse_number::<i32>(indexes[1], line_num)? - 1
                            } else {
                                -1
                            },
                            normal_index: if indexes.len() > 2 {
                                parse_number::<i32>(indexes[2], line_num)? - 1
                            } else {
                                -1
                            },
                        });
                    }
                    surf.faces.push(face);
                }

                ("s", 1) => { // スムーズシェーディングON/OFF
                }

                _ => {
                    // エラー
                    //                  println!("{}[{}]: format error: {}", filename, line_num, text);
                    return Err(format!(
                        "{}[{}]:Format error\n{}\n",
                        filename, line_num, text
                    ));
                }
            }
        }

        if !surf.empty() {
            grp.surfaces.push(surf);
        }

        if !grp.empty() {
            obj.groups.push(grp);
        }

        if !obj.empty() {
            mesh.objects.push(obj);
        }

        return Ok(mesh);
    }

    // マテリアル読み込み
    fn load_mtl(mesh: &mut Mesh<T>, filename: &str) -> Result<bool, String> {
        // ファイルオープン
        let f = match File::open(&filename) {
            Err(_) => {
                return Err(format!("couldn't open {}", filename).to_string());
            }
            Ok(file) => file,
        };

        // オブジェクト準備
        let mut mat = Material::<T>::new();

        // 行単位で読み込み
        let mut line_num: i32 = 0;
        let reader = BufReader::new(f);
        for line in reader.lines() {
            line_num += 1;

            // 空白で分解
            let text: &str = &line.unwrap();
            let params: Vec<&str> = text.trim().split(" ").collect();

            // 空行 or コメント行ならスキップ
            if params.len() <= 0 || params[0].len() <= 0 || params[0].chars().nth(0).unwrap() == '#'
            {
                continue;
            }

            // コマンドと引数に分解
            let command: &str = params[0];
            let args: &[&str] = &params[1..];

            match (command, args.len()) {
                ("newmtl", 1) => {
                    if !mat.empty() {
                        mesh.materials.push(mat);
                        mat = Material::<T>::new();
                    }
                    mat.material_name = args[0].to_string();
                }

                ("Ka", 3) => {
                    mat.ambient.x = parse_number::<T>(args[0], line_num)?;
                    mat.ambient.y = parse_number::<T>(args[1], line_num)?;
                    mat.ambient.z = parse_number::<T>(args[2], line_num)?;
                }

                ("Kd", 3) => {
                    mat.diffuse.x = parse_number::<T>(args[0], line_num)?;
                    mat.diffuse.y = parse_number::<T>(args[1], line_num)?;
                    mat.diffuse.z = parse_number::<T>(args[2], line_num)?;
                }

                ("Ks", 3) => {
                    mat.specular.x = parse_number::<T>(args[0], line_num)?;
                    mat.specular.y = parse_number::<T>(args[1], line_num)?;
                    mat.specular.z = parse_number::<T>(args[2], line_num)?;
                }

                ("Ke", 3) => {}

                ("Ns", 1) => {
                    mat.shininess = parse_number::<T>(args[0], line_num)?;
                }

                ("d", 1) | ("Tr", 1) => {
                    mat.alpha = parse_number::<T>(args[0], line_num)?;
                }

                ("map_Ka", 1) => {
                    mat.ambient_filename = args[0].to_string();
                }

                ("map_Kd", 1) => {
                    mat.diffuse_filename = args[0].to_string();
                }

                ("map_Ks", 1) => {
                    mat.specular_filename = args[0].to_string();
                }

                ("map_Ns", 1) => {}

                ("map_d", 1) => {} // lpha texture map

                ("illum", 1) => {}

                ("map_bump", 1) | ("bump", 1) => {
                    mat.bumpmap_filename = args[0].to_string();
                }

                ("disp", 1) => {}   // displacement map
                ("decal ", 1) => {} // stencil decal texture

                ("Ni", _) => {} //optical density
                ("Tf", _) => {} // Transmission Filter Color

                ("Pr", 1) | ("map_Pr", 1) => {} // roughness
                ("Pm", 1) | ("map_Pm", 1) => {} // metallic
                ("Ps", 1) | ("map_Ps", 1) => {} // sheen
                ("Pc", _) => {}                 // clearcoat thickness
                ("Pcr", _) => {}                // clearcoat roughness
                ("map_ke", 1) => {}             // emissive
                ("aniso", 1) => {}              // anisotropy
                ("anisor", 1) => {}             // anisotropy rotation
                ("norm", 1) => {}               // normal map

                _ => {
                    // エラー
                    //                  println!("{}[{}]: format error: {}", filename, line_num, text);
                    return Err(format!(
                        "{}[{}]:Format error\n{}\n",
                        filename, line_num, text
                    ));
                }
            }
        }

        if !mat.empty() {
            mesh.materials.push(mat);
        }

        return Ok(true);
    }

    pub fn get_vertex_array(&self) -> Vec<T> {
        let mut buffer = Vec::<T>::new();
        for obj in &self.objects {
            for grp in &obj.groups {
                for surf in &grp.surfaces {
                    for face in &surf.faces {
                        for i in 0..3 {
                            assert!(face.points[i].vertex_index >= 0, "");
                            let idx = face.points[i].vertex_index as usize;
                            let vertex = &self.vertexes[idx];
                            buffer.push(vertex.x);
                            buffer.push(vertex.y);
                            buffer.push(vertex.z);

                            if face.points[i].normal_index > 0 {
                                let idx = face.points[i].normal_index as usize;
                                let normal = &self.normals[idx];
                                buffer.push(normal.x);
                                buffer.push(normal.y);
                                buffer.push(normal.z);
                            } else {
                                buffer.push(get::<T>(0.0));
                                buffer.push(get::<T>(0.0));
                                buffer.push(get::<T>(0.0));
                            }

                            if face.points[i].texture_coordinate_index > 0 {
                                let idx = face.points[i].texture_coordinate_index as usize;
                                let texture_coordinate = &self.texture_coordinates[idx];
                                buffer.push(texture_coordinate.u);
                                buffer.push(texture_coordinate.v);
                            } else {
                                buffer.push(get::<T>(0.0));
                                buffer.push(get::<T>(0.0));
                            }
                        }
                    }
                }
            }
        }
        buffer
    }

    pub fn get_surface_info(&self) -> Vec<(i32, i32)> {
        let mut info = Vec::<(i32, i32)>::new();
        for obj in &self.objects {
            for grp in &obj.groups {
                for surf in &grp.surfaces {
                    for face in &surf.faces {
                        info.push((face.points.len() as i32, surf.material_index));
                    }
                }
            }
        }
        info
    }

    pub fn get_matrial(&self, material_index: i32) -> &Material<T> {
        &self.materials[material_index as usize]
    }
}

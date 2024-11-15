use charming::{
    component::{Axis, Grid, Legend, Title},
    element::AxisType,
    series::Line,
    Chart, HtmlRenderer,
};
use data::{
    simple::Simple,
    typed_dataset::{DataRow, TypedDataset},
    urls::Urls,
};
use std::collections::HashMap;

pub mod data;

#[tokio::main]
async fn main() {
    // https://data.overheid.nl/dataset/3795-personenmobiliteit--aandeel-verkeersdeelnemers--persoonskenmerken-2010-2017
    let first = get_data_rows("https://opendata.cbs.nl/ODataApi/OData/83496NED").await;

    // https://data.overheid.nl/dataset/1165-verkeersdeelname-en-deelname-openbaar-vervoer--persoonskenmerken
    let second = get_data_rows("https://opendata.cbs.nl/ODataApi/OData/84707NED").await;

    let combined = first.into_iter().chain(second).collect::<Vec<_>>();

    let chart = create_chart(&combined);

    let mut renderer = HtmlRenderer::new("<Title>", 1100, 680);

    let filename = "chart.html";
    renderer.save(&chart, filename).unwrap();
    eprintln!("saved chart to {filename}");
}

#[allow(clippy::too_many_lines)]
fn create_chart(data: &[DataRow]) -> Chart {
    println!("total rows: {}", data.len());

    let mut by_category: HashMap<u32, Vec<&DataRow>> = HashMap::new();

    for row in data {
        let key = row.perioden.title.parse::<u32>().unwrap();

        by_category.entry(key).or_default().push(row);
    }

    let mut years = by_category.keys().collect::<Vec<_>>();
    years.sort_unstable();
    let years = years.iter().map(ToString::to_string).collect::<Vec<_>>();

    let title_text = "Deelname openbaar vervoer";
    let title_subtext = "Gebruik van het openbaar vervoer\ngeslacht, onderwijsniveau, migratieachtergrond, leeftijd";

    let mut chart = Chart::new()
        .title(Title::new().text(title_text).subtext(title_subtext))
        .legend(Legend::new().left("80%").right("0%"))
        .grid(Grid::new().left("5%").right("65%").top("15%").bottom("50%"))
        .grid(
            Grid::new()
                .left("45%")
                .right("25%")
                .top("15%")
                .bottom("50%"),
        )
        .grid(Grid::new().left("5%").right("65%").top("60%").bottom("5%"))
        .grid(Grid::new().left("45%").right("25%").top("60%").bottom("5%"));

    for i in 0..4 {
        chart = chart
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .name("Year")
                    .data(years.clone())
                    .grid_index(i),
            )
            .y_axis(Axis::new().type_(AxisType::Value).grid_index(i));
    }

    chart = add_to_chart(
        chart,
        &[vec!["Mannen"], vec!["Vrouwen"]],
        |g| {
            filtered_line(&by_category, &format!("Geslacht: {}", g[0]), |row| {
                row.geslacht.title == g[0]
            })
        },
        0,
    );

    let map_persoonskenmerken = |g: &Vec<&str>| {
        filtered_line(&by_category, g[0], |row| {
            g.contains(&row.persoonskenmerken.title.as_str())
        })
    };

    chart = add_to_chart(
        chart,
        &[
            vec!["Actueel onderwijsniveau: laag", "Onderwijsniveau: 1 Laag"],
            vec![
                "Actueel onderwijsniveau: middelbaar",
                "Onderwijsniveau: 2 Middelbaar",
            ],
            vec!["Actueel onderwijsniveau: hoog", "Onderwijsniveau: 3 Hoog"],
        ],
        map_persoonskenmerken,
        1,
    );

    chart = add_to_chart(
        chart,
        &[
            vec!["Migratieachtergrond: Nederland"],
            vec!["Migratieachtergrond: westers"],
            vec!["Migratieachtergrond: niet-westers"],
        ],
        map_persoonskenmerken,
        2,
    );

    add_to_chart(
        chart,
        &[
            vec!["Leeftijd: 12 tot 18 jaar"],
            vec!["Leeftijd: 18 tot 25 jaar"],
            vec!["Leeftijd: 25 tot 35 jaar"],
            vec!["Leeftijd: 35 tot 50 jaar"],
            vec!["Leeftijd: 50 tot 65 jaar"],
            vec!["Leeftijd: 65 tot 75 jaar"],
            vec!["Leeftijd: 75 jaar of ouder"],
        ],
        map_persoonskenmerken,
        3,
    )
}

fn filtered_line(
    by_category: &HashMap<u32, Vec<&DataRow>>,
    name: &str,
    filter: impl Fn(&DataRow) -> bool,
) -> Line {
    let mut sorted = by_category.iter().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|(k, _)| *k);

    Line::new().name(name).data(
        sorted
            .into_iter()
            .map(
                #[allow(clippy::cast_precision_loss)]
                |(_, rows)| {
                    let filtered = rows
                        .iter()
                        .filter(|row| filter(row))
                        .filter_map(|row| row.gebruik_openbaar_vervoer)
                        .collect::<Vec<_>>();

                    filtered.iter().sum::<f64>() / filtered.len() as f64
                },
            )
            .collect(),
    )
}

fn add_to_chart(
    mut chart: Chart,
    categories: &[Vec<&'static str>],
    category_map: impl Fn(&Vec<&str>) -> Line + Copy,
    axis_index: impl Into<f64> + Copy,
) -> Chart {
    let onderwijsniveau = categories.iter().map(category_map);

    for line in onderwijsniveau {
        chart = chart.series(line.x_axis_index(axis_index).y_axis_index(axis_index));
    }

    chart
}

async fn get_data_rows(url: &str) -> Vec<DataRow> {
    eprintln!("fetching data from: {url}");

    let urls = reqwest::get(url)
        .await
        .unwrap()
        .json::<Urls>()
        .await
        .unwrap();

    eprintln!("getting: TypedDataSet");

    let typed_dataset: TypedDataset = get_by_url_key(&urls, "TypedDataSet").await;

    eprintln!("getting: Geslacht");

    let geslacht: Simple = get_by_url_key(&urls, "Geslacht").await;

    eprintln!("getting: Persoonskenmerken");

    let persoonskenmerken: Simple = get_by_url_key(&urls, "Persoonskenmerken").await;

    eprintln!("getting: Perioden");

    let perioden: Simple = get_by_url_key(&urls, "Perioden").await;

    eprintln!("mapping data");

    typed_dataset
        .data
        .into_iter()
        .map(|v| DataRow {
            id: v.id,
            geslacht: geslacht
                .data
                .iter()
                .find(|g| g.key == v.geslacht)
                .unwrap()
                .clone(),
            persoonskenmerken: persoonskenmerken
                .data
                .iter()
                .find(|p| p.key == v.persoonskenmerken)
                .unwrap()
                .clone(),
            perioden: perioden
                .data
                .iter()
                .find(|p| p.key == v.perioden)
                .unwrap()
                .clone(),
            verkeersdeelname: v.verkeersdeelname,
            gebruik_openbaar_vervoer: v.gebruik_openbaar_vervoer,
        })
        .collect()
}

async fn get_by_url_key<T: serde::de::DeserializeOwned>(urls: &Urls, key: &str) -> T {
    reqwest::get(&urls.data.iter().find(|v| v.name == key).unwrap().url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

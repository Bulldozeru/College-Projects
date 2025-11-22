#include "mainwindow.h"
#include "ui_mainwindow.h"
#include "include.h"
#include <QChart>
#include <QChartView>
#include <QFile>
#include <QLineSeries>
#include <QTextStream>
#include <QVBoxLayout>
#include <QtDebug>
#include <QtCharts/QValueAxis>
#include <cmath>
#include "pocketfft_hdronly.h"


using namespace std;
using namespace pocketfft;

string fl_path = "";
double samplefreq = 0;

// Utility: CSV Reader (doesn't need to be a class method)
vector<Sample> readCSV(const string& filename) {

    vector<Sample> data;
    ifstream file(filename);

    if (!file.is_open()) {
        cerr << "Unable to open file: " << filename << endl;
        return data;
    }

    string line;
    getline(file, line); // Skip header

    while (getline(file, line)) {
        stringstream ss(line);
        string field;
        Sample s;

        getline(ss, field, ','); s.time = stod(field);
        getline(ss, field, ','); s.gFx  = stod(field);
        getline(ss, field, ','); s.gFy  = stod(field);
        getline(ss, field, ','); s.gFz  = stod(field);

        data.push_back(s);
    }

    return data;
}
// Utility: Uniform Resampler
vector<double> resampleUniform(const vector<Sample>& data, double rateHz) {
    vector<double> result;

    if (data.empty()) return result;

    double start = data.front().time;
    double end = data.back().time;
    double dt = 1.0 / rateHz;

    size_t j = 0;
    for (double t = start; t <= end; t += dt) {
        while (j + 1 < data.size() && data[j + 1].time < t)
            ++j;

        if (j + 1 < data.size()) {
            double t0 = data[j].time;
            double t1 = data[j + 1].time;
            double v0 = data[j].gFx;
            double v1 = data[j + 1].gFx;

            double interp = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
            result.push_back(interp);
        }
    }

    return result;
}
// FFT Method - now correctly under MainWindow
vector<complex<double>> MainWindow::computeFFT(const vector<double>& signal) {
    size_t N = signal.size();
    vector<complex<double>> out(N);

    pocketfft::shape_t shape = {N};
    pocketfft::stride_t stride_in = {sizeof(double)};
    pocketfft::stride_t stride_out = {sizeof(complex<double>)};
    size_t axis = 0;

    // Perform the FFT with scaling factor 1.0
    pocketfft::r2c(shape, stride_in, stride_out, axis, pocketfft::FORWARD,
                   signal.data(), out.data(), 1.0);

    // Normalize the output by N to prevent magnitude dependence on length
    for (auto& val : out) {
        val /= static_cast<double>(N);
    }

    return out;
}

void MainWindow::plotSpectrum(const vector<complex<double>>& spectrum, double sampleRate) {

    QLineSeries *series = new QLineSeries();
    QLayout *layout = ui->chartContainer->layout();

    if (sampleRate == -1.0) {
        if (layout) {
            QLayoutItem *item;
            while ((item = layout->takeAt(0)) != nullptr) {
                if (item->widget()) {
                    delete item->widget();  // delete chartView
                }
                delete item;
            }
        }
        return;
    }

    int N = spectrum.size() * 2;  // r2c output is N/2 complex values
    for (int i = 0; i < spectrum.size(); ++i) {
        double magnitude = abs(spectrum[i]);
        double frequency = static_cast<double>(i) * sampleRate / N;
        series->append(frequency, magnitude);
    }

    QChart *chart = new QChart();
    QValueAxis *axisX = new QValueAxis();

    series->attachAxis(axisX);
    axisX->setTitleText("Frequency [Hz]");
    axisX->setRange(0, sampleRate / 2);
    axisX->setTickCount(13);

    chart->addSeries(series);
    chart->createDefaultAxes();
    chart->addAxis(axisX, Qt::AlignBottom);
    chart->axes(Qt::Vertical).first()->setTitleText("Magnitude [mm/s*s]");

    QChartView *chartView = new QChartView(chart);
    chartView->setRenderHint(QPainter::Antialiasing);

    if (!layout) {
        layout = new QVBoxLayout(ui->chartContainer);
        ui->chartContainer->setLayout(layout);
    }
    layout->addWidget(chartView);
}

// Constructor
MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow) {
    ui->setupUi(this);
    connect(ui->Enter, &QPushButton::clicked, this, &MainWindow::on_Enter_clicked);
    connect(ui->Clear, &QPushButton::clicked, this, &MainWindow::on_Clear_clicked);
}

// Makes a chart based on inserted values
void MainWindow::on_Enter_clicked() {

    QString fpath = ui->FileEnter->text();
    string path = fpath.toStdString();
    QString hert = ui->HertzData->text();
    bool cond;
    double hertz = hert.toDouble(&cond);
    strpair text;

    text.flpath = fpath;
    text.freq = hertz;
    fl_path = text.flpath.toStdString();
    samplefreq = text.freq;

    if(!cond) {

        ui->ERRORS->setText("[x] ERROR: Could not convert from QString to Double \nLetters are present or the input is empty");
        text.flpath = "";
    }

    if(fpath.size() > 360) {

        ui->ERRORS->setText("[x] ERROR: The path length\n is too big (over 360)");
        text.flpath = "";
    }

    if(hert.size() > 5) {

        ui->ERRORS->setText("[x] ERROR: The number of hertz sample is too big (over 99999)");
        text.flpath = "";
    }

    else {

    auto samples = readCSV(fl_path);
    if (samples.empty()) {
        ui->ERRORS->setText("[x] ERROR: Failed to load or parse CSV data");
        text.flpath = "";
        return;
    }

    ui->generalList->setText(text.flpath + "\n" + hert);
    auto signal = resampleUniform(samples,samplefreq);
    auto spectrum = MainWindow::computeFFT(signal);
    plotSpectrum(spectrum, samplefreq);

    }
}

// Clears the Chart Area
void MainWindow::on_Clear_clicked() {

    vector<complex<double>> blank;
    plotSpectrum(blank, -1);

}

// Destructor
MainWindow::~MainWindow() {
    delete ui;
}



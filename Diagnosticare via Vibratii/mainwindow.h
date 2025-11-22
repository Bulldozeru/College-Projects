#include <QMainWindow>
#include <vector>
#include <complex>

#ifndef MAINWINDOW_H
#define MAINWINDOW_H

using namespace std;

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

struct Sample {
    double time;
    double gFx, gFy, gFz;
};

struct strpair {

    QString flpath;
    double freq;
};


class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();


private slots:
    void on_Clear_clicked();

private:

    Ui::MainWindow *ui;
    void on_Enter_clicked();
    vector<complex<double>> computeFFT(const vector<double>& signal);
    void plotSpectrum(const vector<complex<double>>& spectrum, double sampleRate);

};

#endif // MAINWINDOW_H

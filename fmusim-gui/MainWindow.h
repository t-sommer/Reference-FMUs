#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QLineEdit>
#include <QDoubleValidator>
#include <QComboBox>
#include <QFileSystemModel>
#include <QMap>

QT_BEGIN_NAMESPACE
namespace Ui {
class MainWindow;
}
QT_END_NAMESPACE

extern "C" {
#include "FMIModelDescription.h"
struct FMIRecorder;
}

class ModelVariablesItemModel;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();
    void loadFMU(const QString &filename);
    void setColorScheme(Qt::ColorScheme colorScheme);

protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dropEvent(QDropEvent *event) override;
    void changeEvent(QEvent *event) override;
    // void update

private:
    Ui::MainWindow *ui;
    Qt::ColorScheme colorScheme = Qt::ColorScheme::Dark;
    QLineEdit *stopTimeLineEdit;
    QDoubleValidator *stopTimeValidator;
    QFileSystemModel filesModel;
    QComboBox* interfaceTypeComboBox;
    FMIModelDescription* modelDescription = nullptr;
    QString unzipdir;
    QMap<const FMIModelVariable*, QString> startValues;
    QList<const FMIModelVariable*> plotVariables;
    ModelVariablesItemModel* variablesListModel = nullptr;
    FMIRecorder* recorder = nullptr;

    static void logFunctionCall(FMIInstance* instance, FMIStatus status, const char* message);
    static void logMessage(FMIInstance* instance, FMIStatus status, const char* category, const char* message);

    void setCurrentPage(QWidget *page);
    void updatePlot();

private slots:
    void openFile();
    void selectInputFile();
    void openUnzipDirectory();
    void openFileInDefaultApplication(const QModelIndex &index);
    void simulate();
    void addPlotVariable(const FMIModelVariable* variable);
    void removePlotVariable(const FMIModelVariable* variable);

};
#endif // MAINWINDOW_H

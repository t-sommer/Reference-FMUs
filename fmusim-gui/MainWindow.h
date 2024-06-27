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
}

class ModelVariablesItemModel;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();
    void loadFMU(const QString &filename);

protected:
    void dragEnterEvent(QDragEnterEvent *event) override;
    void dropEvent(QDropEvent *event) override;

private:
    Ui::MainWindow *ui;
    QLineEdit *stopTimeLineEdit;
    QDoubleValidator *stopTimeValidator;
    QFileSystemModel filesModel;
    QComboBox* interfaceTypeComboBox;
    FMIModelDescription* modelDescription = nullptr;
    QString unzipdir;
    QMap<const FMIModelVariable*, QString> startValues;
    ModelVariablesItemModel* variablesListModel = nullptr;

    static void logFunctionCall(FMIInstance* instance, FMIStatus status, const char* message);

    void setCurrentPage(QWidget *page);

private slots:
    void openFile();
    void selectInputFile();
    void openUnzipDirectory();
    void openFileInDefaultApplication(const QModelIndex &index);
    void simulate();

};
#endif // MAINWINDOW_H

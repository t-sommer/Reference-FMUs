#include "MainWindow.h"
#include "./ui_MainWindow.h"
#include <QStringListModel>
#include <QDesktopServices>
#include "ModelVariablesItemModel.h"

extern "C" {
#include "FMIZip.h"
}

#define FMI_PATH_MAX 4096

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    const char* unzipdir = FMICreateTemporaryDirectory();

    this->unzipdir = unzipdir;

    char modelDescriptionPath[FMI_PATH_MAX] = "";

    int status = FMIExtractArchive("C:\\Users\\tsr2\\Downloads\\Reference-FMUs-0.0.31\\3.0\\BouncingBall.fmu", unzipdir);

    FMIPathAppend(modelDescriptionPath, unzipdir);
    FMIPathAppend(modelDescriptionPath, "modelDescription.xml");

    modelDescription = FMIReadModelDescription(modelDescriptionPath); //"C:\\Users\\tsr2\\Downloads\\Reference-FMUs-0.0.31\\3.0\\BouncingBall\\modelDescription.xml");

    switch (modelDescription->fmiVersion) {
    case FMIVersion1:
        ui->FMIVersionLabel->setText("1.0");
        break;
    case FMIVersion2:
        ui->FMIVersionLabel->setText("2.0");
        break;
    case FMIVersion3:
        ui->FMIVersionLabel->setText("3.0");
        break;
    }

    ui->variablesLabel->setText(QString::number(modelDescription->nModelVariables));

    ui->generationDateLabel->setText(modelDescription->generationDate);
    ui->generationToolLabel->setText(modelDescription->generationTool);
    ui->descriptionLabel->setText(modelDescription->description);

    ModelVariablesItemModel* model = new ModelVariablesItemModel(modelDescription, this);

    ui->treeView->setModel(model);

    ui->treeView->setColumnWidth(0, ModelVariablesItemModel::NAME_COLUMN_DEFAULT_WIDTH);
    ui->treeView->setColumnWidth(1, ModelVariablesItemModel::START_COLUMN_DEFAULT_WIDTH);

    filesModel.setRootPath(this->unzipdir);

    auto rootIndex = filesModel.index(this->unzipdir);

    ui->filesTreeView->setModel(&filesModel);
    ui->filesTreeView->setRootIndex(rootIndex);
    ui->filesTreeView->setColumnWidth(0, 200);

    //ui->filesTreeView->resizeColumnToContents(0);
    //ui->filesTreeView->header()->setSectionResizeMode(0, QHeaderView::ResizeToContents);
    connect(ui->filesTreeView, &QAbstractItemView::doubleClicked, this, &MainWindow::openFileInDefaultApplication);
}

MainWindow::~MainWindow()
{
    delete ui;

    if (modelDescription) {
        FMIFreeModelDescription(modelDescription);
    }

    if (!unzipdir.isEmpty()) {
        QByteArray bytes = unzipdir.toLocal8Bit();
        const char *cstr = bytes.data();
        FMIRemoveDirectory(cstr);
    }
}

void MainWindow::openFileInDefaultApplication(const QModelIndex &index) {
    const QString path = filesModel.filePath(index);
    QDesktopServices::openUrl(QUrl(path));
}


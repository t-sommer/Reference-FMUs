#include "MainWindow.h"
#include "./ui_MainWindow.h"
#include <QStringListModel>
#include <QDesktopServices>
#include "ModelVariablesItemModel.h"

extern "C" {
#include "FMIZip.h"
#include "fmusim_fmi3_cs.h"
}

#define FMI_PATH_MAX 4096

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    QIcon::setThemeName("light");

    ui->setupUi(this);

    // add the simulation controls to the toolbar
    stopTimeLineEdit = new QLineEdit(this);
    stopTimeLineEdit->setEnabled(false);
    ui->toolBar->addWidget(stopTimeLineEdit);
    stopTimeLineEdit->setToolTip("Stop time");
    stopTimeLineEdit->setFixedWidth(50);
    stopTimeValidator = new QDoubleValidator(this);
    stopTimeValidator->setBottom(0);
    stopTimeLineEdit->setValidator(stopTimeValidator);

    QWidget* spacer = new QWidget(this);
    spacer->setFixedWidth(10);
    ui->toolBar->addWidget(spacer);

    interfaceTypeComboBox = new QComboBox(this);
    interfaceTypeComboBox->setEnabled(false);
    interfaceTypeComboBox->addItem("Co-Simulation");
    interfaceTypeComboBox->setToolTip("Interface type");
    interfaceTypeComboBox->setSizeAdjustPolicy(QComboBox::AdjustToContents);
    ui->toolBar->addWidget(interfaceTypeComboBox);

    connect(ui->openUnzipDirectoryAction, &QAction::triggered, this, &MainWindow::openUnzipDirectory);
    connect(ui->simulateAction, &QAction::triggered, this, &MainWindow::simulate);

    // hide the dock's title bar
    ui->dockWidget->setTitleBarWidget(new QWidget());

    //ui->filesTreeView->resizeColumnToContents(0);
    //ui->filesTreeView->header()->setSectionResizeMode(0, QHeaderView::ResizeToContents);
    connect(ui->filesTreeView, &QAbstractItemView::doubleClicked, this, &MainWindow::openFileInDefaultApplication);

    connect(ui->showSettingsAction,      &QAction::triggered, this, [this]() { setCurrentPage(ui->settingsPage);     });
    connect(ui->showFilesAction,         &QAction::triggered, this, [this]() { setCurrentPage(ui->filesPage);        });
    connect(ui->showDocumentationAction, &QAction::triggered, this, [this]() { setCurrentPage(ui->documenationPage); });
    connect(ui->showLogAction,           &QAction::triggered, this, [this]() { setCurrentPage(ui->logPage);          });
    connect(ui->showPlotAction,          &QAction::triggered, this, [this]() { setCurrentPage(ui->plotPage);         });
}

void MainWindow::setCurrentPage(QWidget *page) {
    ui->stackedWidget->setCurrentWidget(page);
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

void MainWindow::loadFMU(const QString &filename) {

    const char* unzipdir = FMICreateTemporaryDirectory();

    this->unzipdir = unzipdir;

    char modelDescriptionPath[FMI_PATH_MAX] = "";

    QByteArray bytes = filename.toLocal8Bit();
    const char *cstr = bytes.data();

    int status = FMIExtractArchive(cstr, unzipdir);

    FMIPathAppend(modelDescriptionPath, unzipdir);
    FMIPathAppend(modelDescriptionPath, "modelDescription.xml");

    modelDescription = FMIReadModelDescription(modelDescriptionPath);

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

    // ui->treeView->setColumnWidth(0, ModelVariablesItemModel::NAME_COLUMN_DEFAULT_WIDTH);
    // ui->treeView->setColumnWidth(1, ModelVariablesItemModel::START_COLUMN_DEFAULT_WIDTH);
    const static int COLUMN_WIDTHS[] = {200, 50, 70, 100, 70, 70, 70, 70, 70, 70, 70, 40, 40};

    for (size_t i = 0; i < ModelVariablesItemModel::NUMBER_OF_COLUMNS - 1; i++) {
        ui->treeView->setColumnWidth(i, COLUMN_WIDTHS[i]);
    }

    filesModel.setRootPath(this->unzipdir);

    auto rootIndex = filesModel.index(this->unzipdir);

    ui->filesTreeView->setModel(&filesModel);
    ui->filesTreeView->setRootIndex(rootIndex);
    ui->filesTreeView->setColumnWidth(0, 250);

    const QString doc = QDir::cleanPath(this->unzipdir + QDir::separator() + "documentation" + QDir::separator() + "index.html");
    ui->documentationWebEngineView->load(QUrl::fromLocalFile(doc));

    ui->plotWebEngineView->load(QUrl::fromLocalFile("E:\\Development\\Reference-FMUs\\fmusim-gui\\plot.html"));

    setWindowTitle("FMUSim GUI - " + filename);
}

static void logMessage(FMIInstance* instance, FMIStatus status, const char* category, const char* message) {

    switch (status) {
    case FMIOK:
        printf("[OK] ");
        break;
    case FMIWarning:
        printf("[Warning] ");
        break;
    case FMIDiscard:
        printf("[Discard] ");
        break;
    case FMIError:
        printf("[Error] ");
        break;
    case FMIFatal:
        printf("[Fatal] ");
        break;
    case FMIPending:
        printf("[Pending] ");
        break;
    }

    puts(message);
}

void MainWindow::logFunctionCall(FMIInstance* instance, FMIStatus status, const char* message) {

    MainWindow *window = (MainWindow *)instance->userData;

    QString item(message);


    switch (status) {
    case FMIOK:
        item += " -> OK";
        break;
    case FMIWarning:
        item += " -> Warning";
        break;
    case FMIDiscard:
        item += " -> Discard";
        break;
    case FMIError:
        item += " -> Error";
        break;
    case FMIFatal:
        item += " -> Fatal";
        break;
    case FMIPending:
        item += " -> Pending";
        break;
    default:
        item += " -> Unknown status";
        break;
    }

    window->ui->listWidget->addItem(item);
}

void MainWindow::simulate() {


    FMISimulationSettings settings;

    settings.tolerance                = 0;
    settings.nStartValues             = 0;
    settings.startVariables           = NULL;
    settings.startValues              = NULL;
    settings.startTime                = 0.0;
    settings.outputInterval           = 0.1;
    settings.stopTime                 = 3.0;
    settings.earlyReturnAllowed       = false;
    settings.eventModeUsed            = false;
    settings.recordIntermediateValues = false;
    settings.initialFMUStateFile      = NULL;
    settings.finalFMUStateFile        = NULL;

    char platformBinaryPath[FMI_PATH_MAX] = "";

    auto ba = unzipdir.toLocal8Bit();

    FMIPlatformBinaryPath(ba.data(), modelDescription->coSimulation->modelIdentifier, modelDescription->fmiVersion, platformBinaryPath, FMI_PATH_MAX);

    FMIInstance *S = FMICreateInstance("instance1", logMessage, logFunctionCall);

    if (!S) {
        printf("Failed to create FMU instance.\n");
        return;
    }

    S->userData = this;

    FMILoadPlatformBinary(S, platformBinaryPath);

    //FMIRecorder *recorder = FMICreateRecorder(0, NULL, "BouncingBall_out.csv");

    FMIStatus status = simulateFMI3CS(S, modelDescription, NULL, NULL, NULL, &settings);

    ui->plotWebEngineView->page()->runJavaScript("alert('hello!'); Plotly.newPlot('gd', /* JSON object */ { 'data': [{ 'y': [1, 2, 3] }], 'layout': { 'width': 600, 'height': 400} })");
}

void MainWindow::openUnzipDirectory() {
    QDesktopServices::openUrl(QUrl::fromLocalFile(unzipdir));
}

void MainWindow::openFileInDefaultApplication(const QModelIndex &index) {
    const QString path = filesModel.filePath(index);
    QDesktopServices::openUrl(QUrl(path));
}

